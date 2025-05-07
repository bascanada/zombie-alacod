FROM node:alpine AS builder

# Set the working directory in the container
WORKDIR /app

# Copy package.json and lock file (package-lock.json or yarn.lock)
# This leverages Docker's layer caching. If these files don't change,
# Docker won't re-run npm install/yarn install.
COPY package*.json ./
# If using yarn, uncomment the next line and comment out the npm line below
# COPY yarn.lock ./

# Install dependencies
# If using npm:
RUN npm ci
# If using yarn, uncomment the next line and comment out the npm ci line above
# RUN yarn install --frozen-lockfile

# Copy the rest of your application source code
COPY . .

# Build the SvelteKit application for production
# This command might vary based on your package.json scripts.
# Ensure your SvelteKit project is configured to use adapter-static
# or an adapter that outputs static files (e.g., to a 'build' or '.svelte-kit/output/client' directory).
# For adapter-static, the default output is 'build'.
RUN npm run build


FROM nginx:alpine

WORKDIR /usr/share/nginx/html

RUN rm index.html

COPY --from=builder /app/build /usr/share/nginx/html

EXPOSE 80
