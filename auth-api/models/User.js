const mongoose = require('mongoose');

const UserSchema = new mongoose.Schema({
  username: { type: String, required: true, unique: true },
  password: { type: String, required: true },
  apiKey: { type: String, default: null },
});

module.exports = mongoose.model('User', UserSchema);
