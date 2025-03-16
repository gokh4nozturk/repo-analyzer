# Repo Analyzer API

This is the API service for the Repo Analyzer tool. It handles S3 uploads for repository analysis reports.

## Features

- Secure file uploads to S3
- Authentication via API key
- Configurable S3 bucket and region
- Automatic file cleanup

## Setup

1. Clone the repository
2. Install dependencies:
   ```
   npm install
   ```
3. Copy the example environment file:
   ```
   cp .env.example .env
   ```
4. Edit the `.env` file with your AWS credentials and other settings
5. Create the uploads directory:
   ```
   mkdir uploads
   ```

## Running the API

### Development

```
npm run dev
```

### Production

```
npm start
```

## API Endpoints

### Health Check

```
GET /health
```

Returns a 200 OK response if the API is running.

### Upload File

```
POST /upload
```

Headers:
- `x-api-key`: Your API key (required in production)

Form data:
- `file`: The file to upload (required)
- `bucket`: S3 bucket name (optional, defaults to env variable)
- `key`: S3 object key (optional, generated if not provided)
- `region`: AWS region (optional, defaults to env variable)

Response:
```json
{
  "url": "https://bucket-name.s3.region.amazonaws.com/key",
  "bucket": "bucket-name",
  "key": "key",
  "region": "region"
}
```

## Deployment

The API can be deployed to any Node.js hosting service like:

- AWS Elastic Beanstalk
- Heroku
- Digital Ocean App Platform
- Vercel
- Render

Make sure to set all the required environment variables in your hosting provider's dashboard.

## Security

- API key authentication is enforced in production mode
- File size is limited to 10MB
- Temporary files are automatically cleaned up after upload
- CORS is enabled for all origins (you may want to restrict this in production)

## License

MIT 