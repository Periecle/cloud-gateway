# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the source code to the container
COPY . .

# Build the application in release mode
RUN cargo build --release

# Set the entry point to the application binary
ENTRYPOINT ["/app/target/release/cloud-gateway"]

# Expose the application's port
EXPOSE 8080
