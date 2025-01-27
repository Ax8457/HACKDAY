# Open The Door
Source files to run the challenge locally :

````bash
sudo docker build -t flask-multi-app .
sudo docker run -v /dev/net:/dev/net --cap-add=NET_ADMIN flask-multi-app
````

Front generated with Chat GPT
