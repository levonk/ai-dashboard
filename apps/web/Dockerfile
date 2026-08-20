# AI Dashboard Web Application Dockerfile
# Multi-stage Next.js build for the web app. Build context must be the repo
# root so pnpm workspace deps resolve:
#   docker build -f apps/web/Dockerfile -t ai-dashboard-web .
FROM node:20-alpine AS base
RUN corepack enable pnpm

# Install dependencies for the whole monorepo
FROM base AS deps
RUN apk add --no-cache libc6-compat
WORKDIR /app
COPY package.json pnpm-workspace.yaml ./
COPY apps/web/package.json apps/web/
COPY packages/ui/package.json packages/ui/
# ponytail: no committed pnpm-lock.yaml yet; generate at install time.
# Upgrade path: commit pnpm-lock.yaml and switch back to --frozen-lockfile.
RUN pnpm install --no-frozen-lockfile

# Build the web app
FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/apps/web/node_modules ./apps/web/node_modules
COPY --from=deps /app/packages/ui/node_modules ./packages/ui/node_modules
COPY . .
RUN pnpm --filter web build

# Production image: copy the standalone trace (server.js + minimal node_modules)
FROM node:20-alpine AS runner
WORKDIR /app
ENV NODE_ENV=production
RUN addgroup -g 1001 -S nodejs && adduser -S nextjs -u 1001

# Next.js standalone preserves the monorepo path layout inside dist/standalone.
COPY --from=builder --chown=nextjs:nodejs /app/apps/web/dist/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/apps/web/dist/static ./apps/web/dist/static

USER nextjs
EXPOSE 3000
ENV PORT=3000
ENV HOSTNAME=0.0.0.0
CMD ["node", "apps/web/server.js"]
