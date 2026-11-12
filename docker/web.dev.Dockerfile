# Desenvolvimento do frontend: Vite com HMR, código montado como volume
FROM node:22-bookworm-slim
WORKDIR /app/web
EXPOSE 5173
CMD ["sh", "-c", "npm ci && npm run dev"]
