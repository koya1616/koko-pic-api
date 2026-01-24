.PHONY: build run stop clean

# Build the Docker image
build:
	docker build -t koko-pic-api .

# Run the container
run:
	docker run -d -p 8000:8000 --name koko-pic-api-container koko-pic-api

# Stop and remove the container
stop:
	docker stop koko-pic-api-container || true
	docker rm koko-pic-api-container || true

# Clean up - remove the image
clean: stop
	docker rmi koko-pic-api || true

# Rebuild and restart
restart: clean build run

# View logs
logs:
	docker logs koko-pic-api-container

# Shell into the container
shell:
	docker exec -it koko-pic-api-container /bin/bash