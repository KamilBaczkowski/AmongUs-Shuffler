FROM rust:slim-bullseye AS build

# Make sure the crates.io index is refreshed before things happen, so that this step is properly cached.
RUN cargo search --limit 0

WORKDIR /app
# Install things needed to compile the code.
RUN apt update && apt install lld clang -y
# Copy only information about the dependencies, so that unless those files change, the cache of the
# cargo fetch won't be invalidated unless the dependencies themselves change.
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN cargo fetch
# This is a pre-build to get the dependencies cached.
# The dummy files are there so that the build doesn't fail.
RUN mkdir src
RUN echo "// dummy file" > src/lib.rs && echo 'fn main() { println!("asdf"); }' > src/main.rs
RUN cargo build --release
# Remove the dummy files to prepare for copying the rest of the files.
RUN rm -rf src
# Copy the code to later compile it.
COPY . .
# Make file's modified date later than the previous one, so that cargo has to rebuild.
RUN touch ./src/main.rs ./src/lib.rs
RUN cargo build --release
