# Dropbox

**Important notes**:

- Existing files will be overwritten with new files, be careful with file names;

- **Dropbox** does not provide any security whats so ever!!!

<br>
<br>

**Dropbox** is a simple solution for storing and sharing files.  
It uses **Nginx** for serving static files (insanely fast), and  
a simple custom **Express** server to store uploaded files.  

In general **Dropbox** act as a simple remote hard drive folder   
which can be used to store files with any subfolder structure.  
When `POST` request is done, **Dropbox** uses the request path  
as a path to a folder where file should be placed, all nested  
folders will be created automatically.

<br>

**Dropbox** provides folders navigation out of the box. Just use  
folder name as a url followed by `/`:

![image](https://user-images.githubusercontent.com/62797411/210887358-f8a7bbe1-e1f8-4ffe-97ac-76e4e2a190db.png)

<br>
<br>

## How to use

![image](https://user-images.githubusercontent.com/62797411/210888606-928260da-f019-4b58-88d1-6ea58a7c2401.png)

```
docker run -v $STORAGE_PATH:/data -p $SERVER_PORT:80 tikhoplav/dropbox
```