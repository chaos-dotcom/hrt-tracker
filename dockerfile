# 1. build stage
FROM node:20-alpine AS builder

WORKDIR /app
# copy package files
COPY package.json package-lock.json ./

# install dependencies
RUN npm install

# copy source files
COPY . .

# build
RUN npm run build

# 2. production stage
FROM node:20-alpine AS runner

WORKDIR /app

# copy build output from builder
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/package-lock.json ./package-lock.json

RUN npm install --omit=dev

ENV PORT=3000
EXPOSE 3000

CMD ["/bin/sh", "-c", "mkdir -p /app/data && touch /app/data/hrt-data.json && node build"]
