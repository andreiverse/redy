# Stage 1: Build
FROM node:22-bookworm-slim AS builder
WORKDIR /app

# Install pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
# Nitro/TanStack Start build produces a standalone server in .output/
RUN pnpm build

# Stage 2: Runtime
FROM node:22-bookworm-slim AS runtime
WORKDIR /app

# Copy the build output from the builder stage
# Standard Nitro output is in .output
COPY --from=builder /app/.output .output

# Expose the server port
ENV PORT=3000
EXPOSE 3000

# Run the server
CMD ["node", ".output/server/index.mjs"]
