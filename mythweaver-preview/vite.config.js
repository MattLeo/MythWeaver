export default {
  server: {
    proxy: {
      '/api': {
        target: 'https://api.anthropic.com',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
        headers: {
          'anthropic-version': '2023-06-01',
          'x-api-key': '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',
          'anthropic-dangerous-direct-browser-access': 'true'
        }
      }
    }
  }
}