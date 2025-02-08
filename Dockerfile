FROM rust:1.84.0-slim-bookworm

RUN apt-get update -y && \
    apt-get install -y \
    git \
    curl \
    unzip

# Install protoc
RUN curl https://github.com/protocolbuffers/protobuf/releases/download/v29.3/protoc-29.3-linux-x86_64.zip -Lo protoc.zip
RUN unzip -q protoc.zip bin/protoc 'include/*' -d /usr/local && rm protoc.zip

# Copy sqlc bin
COPY --from=sqlc/sqlc:1.28.0 /workspace/sqlc /usr/bin/sqlc

ARG USERNAME=vscode
ARG GROUPNAME=vscode
ARG UID=1000
ARG GID=1000
RUN groupadd -g $GID $GROUPNAME && \
    useradd -m -s /bin/bash -u $UID -g $GID $USERNAME

USER ${USERNAME}

# Add completions
RUN echo "source /usr/share/bash-completion/completions/git" >> /home/vscode/.bashrc
RUN echo "source <( rustup completions bash )" >> /home/vscode/.bashrc
RUN echo "source <( rustup completions bash cargo )" >> /home/vscode/.bashrc

RUN rustup component add rustfmt clippy
RUN cargo install just