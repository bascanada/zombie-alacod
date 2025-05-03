# Use the official lightweight Nginx image based on Alpine Linux
FROM nginx:alpine

WORKDIR /usr/share/nginx/html

RUN rm index.html

COPY . .

# Expose port 80 to allow incoming HTTP connections
EXPOSE 80

# The base Nginx image already has a command to start Nginx (`CMD ["nginx", "-g", "daemon off;"]`),
# so you don't usually need to specify it again unless you want to override it.
