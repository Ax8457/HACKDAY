# Source - https://stackoverflow.com/a
# Posted by user3905644
# Retrieved 2026-01-15, License - CC BY-SA 3.0

#!/usr/bin/env bash

mongosh mongodb://localhost:27017/hackday <<EOF
db.users.insert({
    username: 'admin_65537',
    password: '51f862031287494838f46b1971039ff7',
    app: ['admin_65537-AUG21'],
    role: 'flag_user',
    company: 'SSRF company',
    last_book: []
})
EOF

mongosh mongodb://localhost:27017/hackday <<EOF
db.appointments.insert({
    reference: 'admin_65537-AUG21',
    event_name: 'Very Secret Meeting',
    event_details: 'Welcome aboard, do not share this with other players ! FLAG => SEFDS0RBWXtCTDFORF9TU1JGX1cxVEhfRjFMVDNSX0JZUDRTU19SMENLU18hISF9 ',
    month: 'AUG',
    day: 21,
    created_by: 'admin_65537',
    members: [ 'admin_65537' ]
})
EOF


