FROM debian:latest

RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    libssl-dev \
    pkg-config \
    sqlite3

# Install latest stable Rust using rustup
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install latest LTS version of Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g npm@latest

RUN cargo install sqlx-cli wasm-bindgen-cli mailtutan

ENV DATABASE_URL=sqlite:///tmp/data.db

# Default command
CMD ["bash"]
