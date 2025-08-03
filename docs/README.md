# Hesha Protocol Documentation Website

This folder contains the GitHub Pages website for the Hesha Protocol.

## Structure

- `index.html` - Landing page
- `documentation.html` - Documentation hub
- `assets/css/style.css` - Website styles
- `assets/images/` - Images and diagrams
- `_config.yml` - GitHub Pages configuration
- Other `.md` files - Protocol specifications and documentation

## Testing Locally

To test the website locally:

```bash
./test-locally.sh
```

Then open http://localhost:8000 in your browser.

## GitHub Pages Setup

1. Go to your repository settings on GitHub
2. Navigate to Pages section
3. Set source to "Deploy from a branch"
4. Choose "main" branch and "/docs" folder
5. Save the settings

The website will be available at: https://heshaorg.github.io/hesha/

## Updating Documentation

When updating documentation:
1. Edit the relevant `.md` files
2. Update `documentation.html` if adding new documents
3. Test locally before pushing
4. Commit and push changes

GitHub Pages will automatically rebuild the site.