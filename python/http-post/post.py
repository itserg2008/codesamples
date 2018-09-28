from urllib.parse import urlencode
from urllib.request import Request, urlopen

url = 'http://localhost:8081/feedback' # Set destination URL here
post_fields = {'name': 'John', 'message': 'Hi there'}     # Set POST fields here

request = Request(url, urlencode(post_fields).encode())
json = urlopen(request).read().decode()
print(json)