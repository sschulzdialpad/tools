use crate::settings::{Repo, Settings};
use crate::util::StatefulHash;

use log::{debug, error, info, warn};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct Status {
    next_page_token: Option<String>,
    items: Option<Vec<StatusItem>>,
}

#[derive(Debug, Deserialize)]
struct StatusItem {
    id: String,
    status: String,
    duration: u16,
    created_at: String,
    stopped_at: String,
    credits_used: u16,
}

pub struct App<'a> {
    pub title: &'a str,
    // TODO: values should be: level = Enum { Cancelled, Failed, .. }
    pub recent: StatefulHash<String, (Repo, String)>,
    pub repos: StatefulHash<Repo, String>,

    poll_delay: HashMap<Repo, u16>,

    client: Client,
    token: String,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, settings: Settings) -> App<'a> {
        let mut poll_delay = HashMap::with_capacity(settings.repos.len());
        for repo in settings.repos.iter() {
            poll_delay.insert(repo.clone(), 0);
        }

        App {
            title,
            recent: StatefulHash::with_items(BTreeMap::new()),
            repos: StatefulHash::with_items(
                settings
                    .repos
                    .iter()
                    .map(|r| (r.clone(), "unknown".to_string()))
                    .collect(),
            ),
            poll_delay: poll_delay,
            client: Client::new(),
            token: settings.token.clone(),
        }
    }

    fn make_request(client: &Client, token: String, repo: &Repo) -> Option<Status> {
        let url = format!(
            "https://circleci.com/api/v2/insights/gh/{}/workflows/{}?branch={}",
            repo.name, repo.workflow, repo.branch
        );
        let request = client
            .get(&url)
            .header("Application", "application/json")
            .header("Circle-Token", token);
        match request.send() {
            Ok(resp) => match resp.json() {
                Ok(body) => Some(body),
                Err(e) => {
                    error!("error decoding CI response json: {:?}", e);
                    None
                }
            },
            Err(e) => {
                error!("error making CI request: {:?}", e);
                None
            }
        }
    }

    fn browse(&mut self) {
        match self.repos.state.selected() {
            Some(i) => match self.repos.items.keys().skip(i).next() {
                Some(repo) => {
                    let chunks = repo.name.split("/").collect::<Vec<_>>();
                    let url = format!(
                        "https://circleci.com/gh/{}/workflows/{}/tree/{}",
                        chunks[0], chunks[1], repo.branch
                    );
                    info!("opening browser to: {}", url);
                    match Command::new("open").arg(url).output() {
                        Ok(_) => (),
                        Err(e) => error!("failed to open browser: {:?}", e),
                    }
                }
                None => error!(
                    "attempted to browse to repo {} of {}",
                    i,
                    self.repos.items.len() - 1
                ),
            },
            None => {
                warn!("attempted to browse to unselected repo");
            }
        }
    }

    pub fn on_key(&mut self, c: char) {
        debug!("keypress: {:?}", c);
        match c {
            '\n' => self.browse(),
            'G' => self.repos.last(),
            'g' => self.repos.first(),
            'j' => self.repos.next(),
            'k' => self.repos.prev(),
            'r' => {
                for (_, val) in self.poll_delay.iter_mut() {
                    *val = 0;
                }
            }
            _ => (),
        }
    }

    pub fn on_tick(&mut self) {
        let mut allow_update = true;
        for (repo, val) in self.poll_delay.iter_mut() {
            if val == &0 {
                if !allow_update {
                    continue;
                }

                // TODO: async then consider multiple updates per tick?
                match Self::make_request(&self.client, self.token.clone(), &repo) {
                    Some(status) => {
                        match status.items {
                            Some(items) if items.len() > 0 => {
                                self.repos
                                    .items
                                    .insert(repo.clone(), items[0].status.clone());
                                for job in items {
                                    self.recent
                                        .items
                                        .insert(job.stopped_at, (repo.clone(), job.status));
                                    if self.recent.items.len() > 5 {
                                        let entry = match self.recent.items.iter().next() {
                                            Some((k, _)) => Some(k.clone()),
                                            _ => None,
                                        };
                                        if entry.is_some() {
                                            self.recent.items.remove(&entry.unwrap());
                                        }
                                    }
                                }
                            }
                            _ => {
                                // TODO: figure out how to grab most recent run from >90 days ago
                                // warn!("got unknown CI status for {}", repo.name);
                                self.repos.items.insert(repo.clone(), "unknown".to_string());
                                ()
                            }
                        }
                        allow_update = false; // be kind to our event loop
                        *val = repo.refresh * 10; // 10 ticks per second
                    }
                    // TODO: should we add a backoff handler?
                    None => (),
                }
                continue;
            }

            *val -= 1;
        }
    }
}
