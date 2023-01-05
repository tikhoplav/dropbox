const fs = require('fs')
const express = require('express')
const busboy = require('busboy')

const app = express()

app.post('*', (req, res) => {
  const path = req.originalUrl

  // Prepare a folder to write a file, if folder exists nothing will happen.
  fs.mkdirSync(`/data${path}`, { recursive: true })

  res.set({
    'Access-Control-Allow-Origin': '*'
  })

  if (req.is('application/octet-stream')) {
    // File extension is going to be taken from the request header, otherwise
    // mime type needs to be resolved from the byte sequence of a file
    const fileName = req.get('file-name')
    if (!fileName) {
      res.writeHead(400, { Connection: 'close' })
      res.write('file name is required')
      return res.end()
    }

    req.on('close', () => {
      res.writeHead(200, { Connection: 'close', Location: `${path}/` })
      return res.end()
    })

    return req.pipe(fs.createWriteStream(`/data${path}/${fileName}`))
  } else {
    // Handle form requests with the `Content-type` header set to
    // `multipart/form-data` or `application/x-www-form-urlencoded`.
    const bb = busboy({
      headers: req.headers,
    })

    bb.on('file', (name, file, info) => {
      const { filename, encoding, mimeType } = info;
      file.pipe(fs.createWriteStream(`/data${path}/${filename}`))
    })

    bb.on('field', (name, val, info) => {
      console.log(`Field [${name}]: value: %j`, val);
    })

    bb.on('close', () => {
      res.writeHead(200, { Connection: 'close', Location: `${path}/` })
      res.end()
    })

    return req.pipe(bb)
  }
})

app.listen(3000, () => {
  console.log(`Dropbox is running on port 3000`)
})