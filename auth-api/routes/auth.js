const express = require('express');
const bcrypt = require('bcrypt');
const jwt = require('jsonwebtoken');
const crypto = require('crypto');
const User = require('../models/User');

const router = express.Router();

// Register
router.post('/register', async (req, res) => {
  try {
    const { username, password } = req.body;
    const existing = await User.findOne({ username });
    if (existing) return res.status(400).json({ error: 'Username already taken' });

    const hashed = await bcrypt.hash(password, 10);
    const user = new User({ username, password: hashed });
    await user.save();

    res.json({ message: 'User registered' });
  } catch (err) {
    console.error('❌ Registration error:', err);
    res.status(500).json({ error: 'Registration failed' });
  }
});

// Login
router.post('/login', async (req, res) => {
  try {
    const { username, password } = req.body;
    console.log(`🔐 Login attempt for: ${username}`);

    const user = await User.findOne({ username });
    if (!user) {
      console.log("❌ User not found");
      return res.status(400).json({ error: 'Invalid credentials' });
    }

    const isMatch = await bcrypt.compare(password, user.password);
    if (!isMatch) {
      console.log("❌ Password mismatch");
      return res.status(400).json({ error: 'Invalid credentials' });
    }

    const token = jwt.sign({ id: user._id }, process.env.JWT_SECRET, { expiresIn: '1d' });
    console.log("✅ Login success!");
    res.json({ token, user: { username: user.username, apiKey: user.apiKey } });
  } catch (err) {
    console.error("❌ Login error:", err);
    res.status(500).json({ error: 'Login failed' });
  }
});

// Middleware to protect routes
const authMiddleware = async (req, res, next) => {
  const token = req.header('Authorization')?.split(' ')[1];
  if (!token) return res.status(401).json({ error: 'Missing token' });

  try {
    const decoded = jwt.verify(token, process.env.JWT_SECRET);
    req.user = await User.findById(decoded.id);
    next();
  } catch {
    res.status(401).json({ error: 'Invalid token' });
  }
};

// Get current user
router.get('/me', authMiddleware, (req, res) => {
  const { username, apiKey } = req.user;
  res.json({
    username,
    apiKey: apiKey || '🔑 No API key yet',
  });
});

// Update API key (generate if blank)
router.put('/api-key', authMiddleware, async (req, res) => {
  try {
    const newKey = req.body.api_key || crypto.randomBytes(16).toString('hex');
    req.user.apiKey = newKey;
    await req.user.save();
    res.json({ message: 'API key updated', apiKey: newKey });
  } catch (err) {
    console.error('❌ Failed to update API key:', err);
    res.status(500).json({ error: 'Could not update API key' });
  }
});

module.exports = router;
