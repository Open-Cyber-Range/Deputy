FROM nexus.ex.c-lab.ee:8003/rust-development:latest

COPY ocr.gpg /usr/ocr.gpg
RUN  apt-key add /usr/ocr.gpg
RUN echo 'deb [arch=amd64] https://nexus.ex.c-lab.ee/repository/ocr-apt-hosted/ focal main' >> /etc/apt/sources.list.d/ocr.list

RUN apt-get update && apt-get upgrade -y
RUN apt-get install deputy -y


RUN mkdir -p /home/vscode/.deputy/download
RUN mkdir /home/vscode/.deputy/index
RUN chown -R vscode:vscode /home/vscode/.deputy

COPY ./deputy /usr/bin/deputy
COPY ./deputy-configuration.toml /home/vscode/.deputy/configuration.toml
