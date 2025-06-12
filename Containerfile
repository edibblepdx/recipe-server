# podman build -t recipe-server .
# podman run -dt --name recipe-server -p 3000:3000 recipe-server:latest

FROM rust:1.87

WORKDIR /usr/src/recipe-server
COPY . .

RUN cargo install --path .

# Remember to expose the port that the application listens on
# with -p 3000:3000
EXPOSE 3000

CMD ["recipe-server", "--host", "0.0.0.0", "--port", "3000"]
