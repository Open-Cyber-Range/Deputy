FROM nexus.ex.c-lab.ee:8003/rust-development:latest

RUN wget -qO - https://nexus.ex.c-lab.ee/repository/ocr-raw-hosted/ubuntu/gpg | apt-key add -
RUN echo 'deb [arch=amd64] https://nexus.ex.c-lab.ee/repository/ocr-apt-hosted/ focal main' >> /etc/apt/sources.list.d/ocr.list

RUN apt-get update && apt-get upgrade -y
RUN apt-get install deputy -y


RUN mkdir -p /home/vscode/.deputy/download
RUN mkdir /home/vscode/.deputy/index
RUN chown -R vscode:vscode /home/vscode/.deputy

COPY ./deputy /usr/bin/deputy
COPY ./deputy-release /usr/bin/deputy-release
COPY ./deputy-configuration.toml /home/vscode/.deputy/configuration.toml
