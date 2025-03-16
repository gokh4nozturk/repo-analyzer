require('dotenv').config();
const express = require('express');
const cors = require('cors');
const multer = require('multer');
const AWS = require('aws-sdk');
const { v4: uuidv4 } = require('uuid');
const fs = require('node:fs');
const path = require('node:path');

// Create Express app
const app = express();
const port = process.env.PORT || 3000;

// Configure middleware
app.use(cors());
app.use(express.json());

// Configure multer for file uploads
const upload = multer({ 
  dest: 'uploads/',
  limits: { fileSize: 10 * 1024 * 1024 } // 10MB limit
});

// Configure AWS
const s3 = new AWS.S3({
  accessKeyId: process.env.AWS_ACCESS_KEY_ID,
  secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  region: process.env.AWS_REGION
});

// Simple API key authentication middleware
const authenticate = (req, res, next) => {
  const apiKey = req.headers['x-api-key'];
  
  // Skip authentication in development mode
  if (process.env.NODE_ENV === 'development') {
    return next();
  }
  
  if (!apiKey || apiKey !== process.env.API_KEY) {
    return res.status(401).json({ error: 'Unauthorized' });
  }
  
  next();
};

// Health check endpoint
app.get('/health', (req, res) => {
  res.status(200).json({ status: 'ok' });
});

// Upload endpoint
app.post('/upload', authenticate, upload.single('file'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No file uploaded' });
    }
    
    // Get parameters from request
    const bucket = req.body.bucket || process.env.AWS_S3_BUCKET;
    const region = req.body.region || process.env.AWS_REGION;
    
    // Generate a key if not provided
    let key = req.body.key;
    if (!key) {
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const filename = req.file.originalname || `report-${timestamp}`;
      key = `reports/${timestamp}-${uuidv4().substring(0, 8)}-${filename}`;
    }
    
    console.log(`Uploading file to S3: ${bucket}/${key}`);
    
    // Read the file
    const fileContent = fs.readFileSync(req.file.path);
    
    // Set up S3 upload parameters
    const params = {
      Bucket: bucket,
      Key: key,
      Body: fileContent,
      ACL: 'public-read',
      ContentType: req.file.mimetype
    };
    
    // Upload to S3
    const data = await s3.upload(params).promise();
    console.log(`File uploaded successfully: ${data.Location}`);
    
    // Clean up the temporary file
    fs.unlinkSync(req.file.path);
    
    // Return the URL
    res.status(200).json({
      url: data.Location,
      bucket,
      key,
      region
    });
  } catch (error) {
    console.error('Error uploading file:', error);
    res.status(500).json({ error: error.message });
  }
});

// Start the server
app.listen(port, () => {
  console.log(`Repo Analyzer API running on port ${port}`);
  console.log(`Upload endpoint: http://localhost:${port}/upload`);
}); 