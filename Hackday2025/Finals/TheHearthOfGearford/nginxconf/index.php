<?php
$session_token = $_COOKIE['session_token'] ?? null;
//fetch API to verify token (return a json) 
function verify_token($token) {
    $url = 'http://127.0.0.1:5000/';
    $ch = curl_init($url);
    curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
    curl_setopt($ch, CURLOPT_COOKIE, "session_token=$token");
    $response = curl_exec($ch);
    curl_close($ch);
    $data = json_decode($response, true);
    if (isset($data['is_valid']) && $data['is_valid'] === true) {
        return $data['is_admin'] ?? false;
    }
    return false;
}

if ($session_token) {
    $is_admin = verify_token($session_token);
    if ($is_admin === true) {
        echo '<p>Hello, you\'re logged in as Admin.</p>';
        echo '<form action="HiddenAdministrativeControlPanel.php">
                <button type="submit">Go to Control Panel</button>
              </form>';
    } else {
        echo '<p>Hello, you\'re logged in as Guest.</p>';
    }
} else {
    echo '<p>You\'re not registered. Please go to <a href="/auth">/auth</a> to get your token.</p>';
}

?>
