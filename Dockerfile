# Use the official Rust image
FROM rust:1.90

# Set the working directory inside the container
WORKDIR /app

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release

# Create a directory for files to be served
RUN mkdir -p /app/files

# Expose the port the app runs on
EXPOSE 3030

# Run the binary
CMD ["./target/release/flash-backend"]