#!/bin/bash

curl -X POST http://localhost:8000/api/v1/add-quote \
  -H "Content-Type: application/json" \
  -d '{
    "id": "1001",
    "quote": "Hard work beats talent when talent doesn’t work hard.",
    "author": "Tim Notke",
    "tags": ["hardwork", "talent"]
  }'

# Helper function to send a quote via curl
post_quote() {
  local id="$1"
  local quote="$2"
  local author="$3"
  local tags="$4"

  curl -s -X POST http://localhost:8000/api/v1/add-quote \
    -H "Content-Type: application/json" \
    -d @- <<EOF
{
  "id": "$id",
  "quote": "$quote",
  "author": "$author",
  "tags": $tags
}
EOF
  echo -e "\n✅ Posted quote ID: $id"
}

# Add quotes
: <<'COMMENT'
post_quote "111" "Do not go where the path may lead, go instead where there is no path and leave a trail." "Ralph Waldo Emerson" '["inspiration", "trail", "path"]'
post_quote "222" "The best way to predict the future is to invent it." "Alan Kay" '["future", "technology", "innovation"]'
post_quote "333" "Life is what happens when you're busy making other plans." "John Lennon" '["life", "plans", "reality"]'
post_quote "444" "The only true wisdom is in knowing you know nothing." "Socrates" '["wisdom", "philosophy", "humility"]'
post_quote "555" "Success is not final, failure is not fatal: It is the courage to continue that counts." "Winston Churchill" '["success", "failure", "courage"]'
post_quote "666" "Whether you think you can or you think you can’t, you’re right." "Henry Ford" '["belief", "attitude"]'
post_quote "777" "What lies behind us and what lies before us are tiny matters compared to what lies within us." "Ralph Waldo Emerson" '["strength", "character", "self"]'
post_quote "888" "In the middle of difficulty lies opportunity." "Albert Einstein" '["challenge", "opportunity"]'
post_quote "999" "Courage is grace under pressure." "Ernest Hemingway" '["courage", "grace"]'
post_quote "1000" "Be yourself; everyone else is already taken." "Oscar Wilde" '["authenticity", "individuality"]'
COMMENT
post_quote "200" "Believe you can and you're halfway there." "Theodore Roosevelt" '["belief", "confidence", "motivation"]'
post_quote "201" "Act as if what you do makes a difference. It does." "William James" '["impact", "action", "purpose"]'
post_quote "202" "What you get by achieving your goals is not as important as what you become by achieving your goals." "Zig Ziglar" '["goals", "growth", "achievement"]'
post_quote "203" "Everything you’ve ever wanted is on the other side of fear." "George Addair" '["fear", "courage", "motivation"]'
post_quote "204" "Limit your always and your nevers." "Amy Poehler" '["mindset", "balance", "moderation"]'
post_quote "205" "Nothing will work unless you do." "Maya Angelou" '["effort", "discipline", "work"]'
post_quote "206" "You are never too old to set another goal or to dream a new dream." "C.S. Lewis" '["dreams", "age", "goals"]'
post_quote "207" "Try to be a rainbow in someone else's cloud." "Maya Angelou" '["kindness", "hope", "positivity"]'
post_quote "208" "We must not allow other people’s limited perceptions to define us." "Virginia Satir" '["identity", "self", "perception"]'
post_quote "209" "You only live once, but if you do it right, once is enough." "Mae West" '["life", "legacy", "living"]'
post_quote "210" "Do what you can, with what you have, where you are." "Theodore Roosevelt" '["action", "resourcefulness", "determination"]'
post_quote "311" "The mind is everything. What you think you become." "Buddha" '["mind", "thoughts", "becoming"]'
post_quote "312" "I have not failed. I've just found 10,000 ways that won't work." "Thomas Edison" '["failure", "persistence", "invention"]'
post_quote "313" "If you want to go fast, go alone. If you want to go far, go together." "African Proverb" '["teamwork", "wisdom", "journey"]'
post_quote "314" "It always seems impossible until it’s done." "Nelson Mandela" '["impossible", "perseverance", "hope"]'
post_quote "315" "Happiness is not something ready made. It comes from your own actions." "Dalai Lama" '["happiness", "action", "mindfulness"]'
post_quote "316" "We are what we repeatedly do. Excellence, then, is not an act, but a habit." "Aristotle" '["excellence", "habit", "discipline"]'
post_quote "317" "A person who never made a mistake never tried anything new." "Albert Einstein" '["mistakes", "innovation", "learning"]'
post_quote "318" "You miss 100% of the shots you don't take." "Wayne Gretzky" '["risk", "opportunity", "sports"]'
post_quote "319" "Strive not to be a success, but rather to be of value." "Albert Einstein" '["value", "success", "purpose"]'
post_quote "320" "Don’t watch the clock; do what it does. Keep going." "Sam Levenson" '["time", "motivation", "persistence"]'
