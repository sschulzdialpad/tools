FROM ubuntu:20.04

ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/GMT
# https://devguide.python.org/#status-of-python-branches
RUN apt-get update -qy && \
    apt-get install -qy --no-install-recommends \
        ca-certificates \
        curl \
        gnupg2 && \
    . /etc/os-release && \
    echo "deb http://ppa.launchpad.net/deadsnakes/ppa/ubuntu ${UBUNTU_CODENAME} main" > /etc/apt/sources.list.d/deadsnakes.list && \
    apt-key adv --keyserver keyserver.ubuntu.com --recv-keys F23C5A6CF475977595C89F51BA6932366A755776 && \
    apt-get update -qy && \
    apt-get install -qy --no-install-recommends \
        git \
        openssh-client \
        python2.7 \
        python2.7-dev \
        python3.5 \
        python3.5-dev \
        python3.6 \
        python3.6-dev \
        python3.7 \
        python3.7-dev \
        python3.8 \
        python3.8-dev \
        python3.9 \
        python3.9-dev \
        python3.9-distutils \
        python3.10 \
        python3.10-dev \
        python3.10-distutils \
        python3-pip && \
    curl -fsS https://bootstrap.pypa.io/pip/2.7/get-pip.py --output /tmp/get-pip-2.7.py && \
    python2.7 /tmp/get-pip-2.7.py && \
    rm /tmp/get-pip-2.7.py && \
    python2.7 -m pip install --no-cache-dir --upgrade pip && \
    python3.5 -m pip install --no-cache-dir --upgrade pip && \
    python3.6 -m pip install --no-cache-dir --upgrade pip && \
    python3.7 -m pip install --no-cache-dir --upgrade pip && \
    python3.8 -m pip install --no-cache-dir --upgrade pip && \
    python3.9 -m pip install --no-cache-dir --upgrade pip && \
    curl -fsS https://bootstrap.pypa.io/get-pip.py --output /tmp/get-pip.py && \
    python3.10 /tmp/get-pip.py && \
    rm /tmp/get-pip.py && \
    python3.10 -m pip install --no-cache-dir --upgrade pip && \
    python3 -m pip install --no-cache-dir 'nox==2022.1.7' && \
    rm -rf /var/cache/apt/lists
