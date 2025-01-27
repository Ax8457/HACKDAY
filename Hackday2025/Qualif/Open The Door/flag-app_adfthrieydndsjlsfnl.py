from flask import Flask, render_template
import os

app = Flask(__name__)

@app.route('/')
def render_html():
    return render_template('flag-djiuhehfcnfliezhfch.html')

if __name__ == '__main__':
    app.run(debug=False, host='0.0.0.0', port=14456)
