const express = require('express');
const bodyParser = require('body-parser');
const mongoose = require('mongoose');
const session = require('express-session'); 
const { v4: uuidv4 } = require('uuid');
const app = express();
const { firefox } = require('playwright');

app.use(session({
    secret: 'supersecretazerty', 
    resave: false, 
    saveUninitialized: true, 
    cookie: { secure: false }
}));

let publishedTokens = []; 

async function checkTokensPeriodically() {
    console.log('Checking profiles Init');
    setInterval(async () => {
        if (publishedTokens.length > 0) {
            console.log('Checking profiles');
            await checkProfiles(publishedTokens); 
        } else {
            console.log('No token remaining.');
        }
    }, 30000); 
}

async function checkProfiles() {
    const cookie = {
        name: 'FLAG',
        value: 'HACKDAY{0rd3R_M4tteRs_In_Ur_C0d3!!!!}',
        domain: 'localhost',
        path: '/',
        httpOnly: false,
        secure: false,
    };
    let browser;
    browser = await firefox.launch({
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox'],
    });
    const page = await browser.newPage();
    await page.context().addCookies([cookie]);
    const cookies = await page.context().cookies();
    console.log('Cookies defined :', cookies);
    for (let i = 0; i < publishedTokens.length; i++) {
        const token = publishedTokens[i];
        try {
            const profileUrl = `http://localhost:3000/user?token=${token}`;
            await page.goto(profileUrl);
            const profile = await Profile.findOneAndUpdate(
                { token: token }, 
                { $set: { 'profileData.isVerified': true } },
                { new: true }
            );
            console.log(`User ${token}: Checked`);
            publishedTokens.splice(i, 1);
            console.log(`Token ${token} deleted.`);
            i--; 
        } catch (error) {
            console.error(`Error while checking profile ${token}:`, error);
        }
    }
    console.log('Remaining Token:', publishedTokens);
    await browser.close();
}

async function clearDatabaseAndArticlesPeriodically() {
    console.log('Database and Articles Reset Init');
    setInterval(async () => {
        try {
            console.log('Clearing database and resetting articles...');
            await Profile.deleteMany({});
            console.log('MongoDB has been cleared.');
            articles = [];
            console.log('Articles list has been reset.');
        } catch (err) {
            console.error('Error while clearing database:', err);
        }
    }, 900000); 
}

function sanitizeJson(jsonInput, res) {
    try {
        const jsonData = typeof jsonInput === "string" ? JSON.parse(jsonInput) : jsonInput;
        function containsXSSChars(data) {
            const xssChars = /[<>/\"'`&#()}{#]/;
            if (typeof data === "string") {
                return xssChars.test(data);
            } else if (Array.isArray(data)) {
                return data.some(item => containsXSSChars(item));
            } else if (typeof data === "object" && data !== null) {
                return Object.values(data).some(value => containsXSSChars(value));
            }
            return false;
        }
        if (containsXSSChars(jsonData)) {
            return res.status(500).send('XSS detected !!!!!');
        }
    } catch (error) {
        console.error("Erreur :", error);
        return res.status(500).send('JSON Error');
    }
}

async function checkToken(token) {
    try {
        console.log('Token received :', token);
        const profile = await Profile.findOne({ token });
        console.log(profile);
        if (!profile) {
            console.error(`Token not valid or not found: ${token}`);
            return { error: 'Token isnt valid.' };
        }
        const username = profile.profileData.get('username'); 
        const bool = profile.profileData.get('isVerified'); 
        console.log(bool);
        console.log('User found :', username);
        return { username: username, bool: bool };
    } catch (err) {
        console.error('Error while checking token :', err);
        return { error: 'Internal Error.' };
    }
}

mongoose.connect('mongodb://db:27017/blog', {
    useNewUrlParser: true,
    useUnifiedTopology: true,
}, (err) => {
    if (err) {
        console.log('Error while connecting to MongoDB :', err);
    } else {
        console.log('Successfully connected to MongoDB');
    }
});

const profileSchema = new mongoose.Schema({
    token: { type: String, required: true, unique: true },
    id: { type: String, required: true, unique: true },
    profileData: { 
        type: Map, 
        of: mongoose.Schema.Types.Mixed, 
        required: true 
    },
    messages: [
        {
            message: { type: String, required: true }
        }
    ],
    createdAt: { type: Date, default: Date.now },
});

const Profile = mongoose.model('Profile', profileSchema);
let articles = [];

app.set('view engine', 'ejs');
app.use(express.static('public'));
app.use(bodyParser.urlencoded({ extended: false }));

app.get('/', (req, res) => {
    res.render('index', { articles });
});

app.get('/new', (req, res) => {
    res.render('new');
});

app.post('/articles', async (req, res) => {
    const { title, content, authorToken } = req.body;
    const { username, bool, error } = await checkToken(authorToken); 
    console.log(bool);
    if (error) {
        return res.status(400).send(error); 
    }
    if (!publishedTokens.includes(authorToken)) {
        publishedTokens.push(authorToken);
        console.log(`Token added to checklist : ${authorToken}`);
    }
    if (title.includes("HACKDAY{0rd3R_M4tteRs_In_Ur_C0d3!!!!}") || content.includes("HACKDAY{0rd3R_M4tteRs_In_Ur_C0d3!!!!}")) {
        return res.status(400).send("Title or content cannot contain the secret data.");
    }
    articles.push({ title, content, author: username, bool , createdAt: new Date() });
    console.log(`Article released by ${username} :`, { title, content });
    res.redirect('/');
});

app.get('/profile', (req, res) => {
    req.session.tempToken = uuidv4();  
    req.session.tempId = uuidv4();  
    res.render('profile', { 
        token: req.session.tempToken, 
        id: req.session.tempId 
    });
});


app.post('/profile', async (req, res) => {
    const { token, id, username, name, description, place } = req.body;
    const sessionToken = req.session.tempToken;  
    const sessionId = req.session.tempId; 

    if (!token || !id || !username || !name || !description || !place) {
        return res.status(400).send('Please complete all the fields.');
    }

    if (token !== sessionToken) { 
        return res.status(400).send('Wrong token, use the one printed!');
    }

    const profileData = {
        username,
        name,
        description,
        place,
        isVerified: false
    };

    req.session.profile = { ...profileData, token, id }; 
    try {
        const existingProfile = await Profile.findOne({ token });
        if (existingProfile) {
            return res.status(400).send('Error, token already used.');
        }

        const newProfile = new Profile({
            token,
            id,
            profileData,
        });
        await newProfile.save();

        const isXSSDetected = sanitizeJson(profileData, res);
        if (isXSSDetected) return;

        console.log('Profile created :', profileData);
        res.redirect(`/user?token=${token}`);
    } catch (err) {
        console.error('Error while creating profile :', err);
        res.status(500).send('Error while creating profile.');
    }
});

app.get('/user', async (req, res) => {
    const token = req.query.token; 
    if (!token) {
        return res.status(400).send('Token required.');
    }
    try {
        const profile = await Profile.findOne({ token });
        if (!profile) {
            return res.status(404).send('Profile not found.');
        }

        res.render('user', { profile, sessionData: req.session.profile }); 
    } catch (err) {
        console.error('Error while retrieving profile :', err);
        res.status(500).send('Error while retrieving profile.');
    }
});

app.get('/message', (req, res) => {
    res.render('message');
});

app.post('/sendmessage', async (req, res) => {
    const { contact, message } = req.body; 
    if (!contact || !message) {
        return res.status(400).send('Both contact ID and message are required.');
    }
    try {
        const user = await Profile.findOne({ id: contact });
        if (!user) {
            return res.status(404).send('User not found.');
        }
        const newMessage = { message };
        const updatedProfile = await Profile.findOneAndUpdate(
            { id: contact },
            { $push: { messages: newMessage } }, 
            { new: true }
        );
        console.log(`Message sent to user ${contact}:`, message);
        res.status(200).send({ message: 'Message sent successfully.'});
    } catch (error) {
        console.error('Error while sending message:', error);
        res.status(500).send('Internal server error.');
    }
});

const PORT = 3000;
app.listen(PORT, () => {
    console.log(`Server running => http://localhost:${PORT}`);
    checkTokensPeriodically();
    clearDatabaseAndArticlesPeriodically(); 
});

