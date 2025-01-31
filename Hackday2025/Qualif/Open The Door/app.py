from flask import Flask, render_template_string, request, send_file
from reportlab.lib.pagesizes import letter
from reportlab.pdfgen import canvas
import os

app = Flask(__name__, static_url_path='/', static_folder='/')
SAMPLES_DIR = 'samples'
PDF_DIR = 'PDF'
DESC_DIR = 'samples_desc'

def remove_dotdot_slash(input_string):
    banned_chars = ['../', '%', '<', '>', '#','_','-','|','{','}']
    for char in banned_chars:
        input_string = input_string.replace(char, '')
    return input_string

@app.route('/')
def home():
    files = [f for f in os.listdir(SAMPLES_DIR) if os.path.isfile(os.path.join(SAMPLES_DIR, f))]
    file_cards = ""
    for file in files:
        file_cards += f'''
        <div class="card">
            <h3>{file}</h3>
            <img src="/samples/{file}" alt="{file}" />
        </div>
        '''
    return render_template_string('''
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Weapons Plans</title>
        <style>
            body {
                font-family: 'Arial', sans-serif;
                margin: 0;
                padding: 50px 0;
                display: flex;
                justify-content: center;
                align-items: center;
                flex-direction: column;
                min-height: 100vh;
                background-color: #2c3e50;
                background-image: linear-gradient(to right, rgba(255,255,255,0.1) 1px, transparent 1px), 
                                  linear-gradient(to bottom, rgba(255,255,255,0.1) 1px, transparent 1px);
                background-size: 40px 40px;
                color: white;
            }
            h1, h2 {
                text-align: center;
                margin: 30px 0;
                font-size: 2.5rem;
                font-weight: bold;
                text-shadow: 2px 2px 4px rgba(0,0,0,0.7);
            }
            .container {
                display: flex;
                flex-wrap: wrap;
                justify-content: center;
                gap: 20px;
                padding: 20px;
                box-sizing: border-box;
                width: 100%;
                max-width: 1200px;
            }
            .card {
                background-color: rgba(255, 255, 255, 0.9);
                border-radius: 12px;
                overflow: hidden;
                box-shadow: 0px 4px 10px rgba(0, 0, 0, 0.2);
                width: 280px;
                transition: transform 0.3s, box-shadow 0.3s;
                text-align: center;
                cursor: pointer;
            }
            .card:hover {
                transform: scale(1.05);
                box-shadow: 0px 6px 15px rgba(0, 0, 0, 0.3);
            }
            .card img {
                width: 100%;
                height: 200px;
                object-fit: cover;
                border-bottom: 1px solid #ddd;
            }
            .card h3 {
                margin: 15px 0;
                font-size: 1.1rem;
                color: #333;
            }
            .button-container {
                margin-top: 50px;
            }
            .button-container a {
                text-decoration: none;
                padding: 10px 20px;
                background-color: #e74c3c;
                color: white;
                font-size: 1.2rem;
                border-radius: 5px;
                transition: background-color 0.3s ease;
            }
            .button-container a:hover {
                background-color: #c0392b;
            }
        </style>
    </head>
    <body>
        <h1>Weapons Plans</h1>
        <h2>Choose your plan</h2>
        <div class="container">
            {{ file_cards|safe }}
        </div>
        <div class="button-container">
            <a href="/download">Go to Download Page</a>
        </div>
    </body>
    </html>
    ''', file_cards=file_cards)

@app.route('/download', methods=['GET', 'POST'])
def download_page():
    files = [f for f in os.listdir(SAMPLES_DIR) if os.path.isfile(os.path.join(SAMPLES_DIR, f))]
    desc_files = [f for f in os.listdir(DESC_DIR) if os.path.isfile(os.path.join(DESC_DIR, f))]
    desc_data = []
    
    for desc_file in desc_files:
        with open(os.path.join(DESC_DIR, desc_file), 'r') as f:
            desc_data.append({"name": desc_file, "content": f.read()})

    if request.method == 'POST':
        selected_file = request.form.get('file')
        desc_file = request.form.get('desc_file')
        selected_file = remove_dotdot_slash(selected_file)
        desc_file = remove_dotdot_slash(desc_file)
        selected_file_path = os.path.join(SAMPLES_DIR, selected_file)
        desc_file_path = os.path.join(DESC_DIR, desc_file)
      
        if not selected_file or not desc_file:
            return "Error: Both file and description file name must be provided.", 400

        if os.path.exists(selected_file_path) and os.path.exists(desc_file_path):
            with open(desc_file_path, 'r') as desc:
                description_content = desc.read()

            pdf_path = os.path.join(PDF_DIR, f'{os.path.splitext(selected_file)[0]}.pdf')
            c = canvas.Canvas(pdf_path, pagesize=letter)
            c.setFont("Helvetica-Bold", 20)
            c.drawString(100, 750, f"Title: {selected_file}")
            c.setFont("Helvetica", 12)
            c.drawString(100, 720, "Description:")
            c.setFont("Helvetica", 10)
            text_object = c.beginText(100, 700)
            for line in description_content.splitlines():
                text_object.textLine(line)
            c.drawText(text_object)
            c.drawImage(selected_file_path, 100, 300, width=400, height=300)
            c.save()

            return send_file(pdf_path, as_attachment=True)
        else:
            return f"Error: File '{selected_file}' or '{desc_file}' not found.", 404

    return render_template_string('''
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Download Page</title>
        <style>
           body {
                font-family: 'Arial', sans-serif;
                margin: 0;
                padding: 100px 0;
                display: flex;
                justify-content: center;
                align-items: center;
                flex-direction: column;
                min-height: 100vh;
                background-color: #2c3e50;
                background-image: linear-gradient(to right, rgba(255,255,255,0.1) 1px, transparent 1px), 
                                  linear-gradient(to bottom, rgba(255,255,255,0.1) 1px, transparent 1px);
                background-size: 40px 40px;
                color: white;
            }
            table {
                width: 90%;
                margin: 20px auto;
                border-collapse: collapse;
                background-color: rgba(255, 255, 255, 0.1);
                color: white;
            }
            th, td {
                padding: 15px;
                border: 1px solid white;
                text-align: left;
            }
            th {
                background-color: rgba(231, 76, 60, 0.8);
                color: white;
            }
            button{
                padding: 10px 20px;
                font-size: 16px;
                color: white;
                background-color: #e74c3c;
                border: none;
                border-radius: 5px;
                cursor: pointer;
                transition: background-color 0.3s ease;
                margin-top: 10px;
            }
            input{
                padding: 10px 20px;
                font-size: 16px;
                color: black;
                background-color: white;
                border: none;
                border-radius: 5px;
                cursor: pointer;
                transition: background-color 0.3s ease;
                margin-top: 10px;
            }
            button:hover, select:hover {
                background-color: #c0392b;
            }
        </style>
    </head>
    <body>
        <h1>Available Files</h1>
        <table>
            <thead>
                <tr>
                    <th>File Name</th>
                    <th>Preview</th>
                </tr>
            </thead>
            <tbody>
                {% for file in files %}
                <tr>
                    <td>{{ file }}</td>
                    <td><img src="/samples/{{ file }}" alt="{{ file }}" style="max-height: 100px;"></td>
                </tr>
                {% endfor %}
            </tbody>
        </table>

        <h1>Additional Files Available</h1>
        <table>
            <thead>
                <tr>
                    <th>File Name</th>
                    <th>Content</th>
                </tr>
            </thead>
            <tbody>
                {% for desc in desc_data %}
                <tr>
                    <td>{{ desc.name }}</td>
                    <td>{{ desc.content }}</td>
                </tr>
                {% endfor %}
            </tbody>
        </table>

        <form method="POST">
    <input type="text" name="file" placeholder="Enter the plan file name (e.g., weapon.png)" required>
    <input type="text" name="desc_file" placeholder="Enter the description file name (e.g., description.txt)" required>
    <button type="submit">Generate PDF</button>
	</form>
    </body>
    </html>
    ''', files=files, desc_data=desc_data)


if __name__ == '__main__':
    app.run(debug=False, host='0.0.0.0', port=5000)

