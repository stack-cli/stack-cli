# my-rari-app

A high-performance React Server Components application powered by [rari](https://rari.dev).

## 🚀 Getting Started

```bash
# Install dependencies
{{INSTALL_COMMAND}}

# Start development server
pnpm run dev
```

Visit [http://localhost:5173](http://localhost:5173) to see your app.

## 🚀 Deploy to the Cloud

This rari application is pre-configured for cloud deployment.

### 🚂 Railway

### Quick Deploy

1. **Push to GitHub**:
   ```bash
   git add .
   git commit -m "Initial commit"
   git push origin main
   ```

2. **Deploy to Railway**:
   - Go to [railway.app](https://railway.app)
   - Create new project → "Deploy from GitHub repo"
   - Select your repository
   - Click "Deploy Now"

3. **Generate Domain**:
   - In Railway dashboard → Settings → Networking
   - Click "Generate Domain"
   - Your app will be live! 🎉

### Alternative: Setup Railway from CLI

```bash
# Configure Railway deployment files
pnpm run deploy:railway

# Follow the instructions to deploy
```

### 🎨 Render

1. **Push to GitHub**:
   ```bash
   git add .
   git commit -m "Initial commit"
   git push origin main
   ```

2. **Deploy to Render**:
   - Go to [render.com](https://render.com)
   - Create new "Web Service"
   - Connect your GitHub repository
   - Render auto-detects Node.js and uses `render.yaml`
   - Click "Create Web Service"

### Alternative: Setup Render from CLI

```bash
# Configure Render deployment files
pnpm run deploy:render

# Follow the instructions to deploy
```

## 📜 Available Scripts

```bash
# Development
pnpm run dev          # Start development server
pnpm run build        # Build for production

# Production
pnpm start            # Start production server
pnpm run start:local  # Start local production server

# Deployment
pnpm run deploy:railway  # Setup Railway deployment
pnpm run deploy:render   # Setup Render deployment

# Code Quality
pnpm run lint         # Run linters
pnpm run typecheck    # Run TypeScript checks
```

## 🌍 Environment Variables

Cloud platforms automatically provide:
- `PORT` - Server port (platform assigns this)
- `NODE_ENV=production` - Production mode

Optional variables you can set:
- `RUST_LOG=debug` - Rust logging level

## 🏗️ Architecture

- **⚡ Rust Runtime**: Native performance with zero-cost abstractions
- **🚀 React Server Components**: True server-side rendering
- **📁 File-based Routing**: Automatic route generation
- **🎯 Zero Configuration**: Works out of the box

## 📚 Learn More

- [rari Documentation](https://rari.dev)
- [Railway Documentation](https://docs.railway.app)
- [Render Documentation](https://render.com/docs)
- [React Server Components](https://react.dev/reference/react/use-server)

---

Built with ❤️ using [rari](https://rari.dev)
