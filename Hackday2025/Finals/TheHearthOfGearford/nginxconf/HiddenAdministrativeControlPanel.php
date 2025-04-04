<?php

//function which check token before rendering page
function verify_token($token) {
    $url = 'http://127.0.0.1:5000/'; //an API deployed on the localhost , not exposed can't be fetched, return a json
    $ch = curl_init($url);
    curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
    curl_setopt($ch, CURLOPT_COOKIE, "session_token=$token");
    $response = curl_exec($ch);
    curl_close($ch);
    $data = json_decode($response, true);
    if (isset($data['is_valid']) && $data['is_valid'] === true && isset($data['is_admin']) && $data['is_admin'] === true) {
        return true; 
    } else {
        return false; 
    }
}

$session_token = $_COOKIE['session_token'] ?? null;

if ($session_token) {
    if (verify_token($session_token)) {

        //collect logs
        function get_logs($logfile = '/var/log/auth.log', $lines = 20) { 
            if (!file_exists($logfile)) {
                return "The log file does not exist.";
            }
            if (!is_readable($logfile)) {
                return "The log file is not readable.";
            }
            $output = shell_exec("tail -n $lines $logfile");
            if ($output === null) {
                return "Error executing the shell command.";
            }
            $log_lines = explode("\n", $output);
            $formatted_logs = [];
            foreach ($log_lines as $line) {
                if (!empty($line)) {
                    $formatted_logs[] = format_log_line($line);
                }
            }
            return implode('<br>', $formatted_logs);
        }

        //reshape logs
        function format_log_line($line) {
            $pattern = '/^(\w+\s+\d+\s+\d+:\d+:\d+:\d+)\s+(\S+)\s+(\S+)\[(\d+)\]:\s+(.*)$/';
            if (preg_match($pattern, $line, $matches)) {
                return "<strong>Date:</strong> $matches[1] <br>
                        <strong>Machine:</strong> $matches[2] <br>
                        <strong>Service:</strong> $matches[3] <br>
                        <strong>PID:</strong> $matches[4] <br>
                        <strong>Message:</strong> $matches[5] <br><br>";
            }
            return "<pre>$line</pre>";
        }

        // get log associated to specific ID
        function get_log_by_id($logfile = '/var/log/auth.log', $log_id) {
            if (!file_exists($logfile) || !is_readable($logfile)) {
                return "Error reading log file.";
            }
            $output = shell_exec("tail -n 20 $logfile"); //last 20 lines of auth.log
            $log_lines = explode("\n", $output);
            return isset($log_lines[$log_id - 1]) ? format_log_line($log_lines[$log_id - 1]) : "Log not found.";
        }

        // load file feature
        function load_external_log($logfile = '') {
            return isset($logfile) && !empty($logfile) ? include($logfile) : "No file specified.";
        }

        // gather SSH users who failed logging in 
        function get_ssh_users($logfile = '/var/log/auth.log') {
            if (!file_exists($logfile) || !is_readable($logfile)) {
                return "Error reading log file.";
            }
            $output = shell_exec("tail -n 20 $logfile"); //only collect the users in the last 20 lines of auth.log 
            if ($output === null) {
                return "Error executing the shell command.";
            }
            preg_match_all('/sshd\[\d+\]:\s+Failed password for invalid user (\S+)/', $output, $matches);
            $unique_users = array_unique($matches[1]);
            if (empty($unique_users)) {
                return "No SSH failed login attempts found.";
            }
            file_put_contents('/var/www/html/userloginfail.txt', implode("\n", $unique_users));
            return "<ul><li>" . implode("</li><li>", $unique_users) . "</li></ul><p><small>Failed SSH users are stored in <code>userloginfail.txt</code>.</small></p>";
        }

        // clear ssh users file => important for the exploit
        function clear_failed_users() {
            $file_path = '/var/www/html/userloginfail.txt';
            return file_exists($file_path) ? file_put_contents($file_path, '') . "The failed SSH user list has been cleared." : "No failed SSH user list to clear.";
        }
        ?>

        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Hidden Administrative Control Panel</title>
            <link rel="stylesheet" href="style.css">
        </head>
        <body>

        <div class="container">
            <div class="banner">
                <h1>Hidden Administrative Control Panel</h1>
            </div>

            <div class="panel">
                <button class="panel-button" onclick="showPanel('auth')">Auth Logs</button>
                <button class="panel-button" onclick="showPanel('network')">Network Traffic</button>
                <button class="panel-button" onclick="showPanel('notes')">Notes</button>
                <button class="panel-button" onclick="showPanel('log-detail')">Log Detail 👁️</button>
                <button class="panel-button" onclick="showPanel('load-log')">Load External Log</button>
                <button class="panel-button" onclick="showPanel('ssh-users')">SSH Users</button>
            </div>

            <div id="auth" class="panel-content">
                <h3>Authentication Logs</h3>
                <?php echo get_logs('/var/log/auth.log', 20); ?>
            </div>

            <div id="network" class="panel-content">
                <h3>Network Traffic</h3>
                <p>Network traffic statistics will be displayed here soon...</p>
            </div>

            <div id="notes" class="panel-content">
                <h3>Notes</h3>
                <form action="" method="POST">
                    <textarea name="note" placeholder="Enter your note here..."></textarea>
                    <button type="submit">Save Note</button>
                </form>
                <?php 
                    $notes = file('/var/www/html/notes.txt', FILE_IGNORE_NEW_LINES);
                    foreach ($notes as $note) : 
                ?>
                    <div class="note"><p><?= htmlspecialchars($note) ?></p></div>
                <?php endforeach; ?>
            </div>

            <div id="log-detail" class="panel-content">
                <h3>Log Detail</h3>
                <form action="" method="GET">
                    <label for="log-id">Enter Log ID:</label>
                    <input type="number" name="log" id="log-id" placeholder="Log ID" required>
                    <button type="submit">View Log</button>
                </form>
                <?php
                    if (isset($_GET['log']) && !empty($_GET['log'])) {
                        echo "<div class='log-wrapper'>" . get_log_by_id('/var/log/auth.log', (int)$_GET['log']) . "</div>";
                    }
                ?>
            </div>

            <div id="load-log" class="panel-content">
                <h3>Load other file Log</h3>
                <form action="" method="GET">
                    <label for="load-id">Enter External Log File:</label>
                    <input type="text" name="load" id="load-id" placeholder="log file" required>
                    <button type="submit">Load Log</button>
                </form>
                <?php
                    if (isset($_GET['load']) && !empty($_GET['load'])) {
                        echo load_external_log($_GET['load']);
                    }
                ?>
            </div>

            <div id="ssh-users" class="panel-content">
                <h3>SSH Failed Login Users (20 last lines of auth.log)</h3>
                <?php echo get_ssh_users('/var/log/auth.log'); ?>
                <form action="" method="POST">
                    <button type="submit" name="clear-users">Clear SSH Users file</button>
                </form>
                <?php if (isset($_POST['clear-users'])) echo clear_failed_users(); ?>
            </div>
        </div>

        <script>
            function showPanel(panelId) {
                document.querySelectorAll('.panel-content').forEach(panel => panel.style.display = 'none');
                document.getElementById(panelId).style.display = 'block';
            }
            showPanel('auth');
        </script>
        </body>
        </html>
        <?php
    } else {
        echo "Invalid or non-admin token.";
    }
} else {
    echo "No session token found.";
}
?>
