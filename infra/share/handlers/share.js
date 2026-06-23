const { S3Client, PutObjectCommand, GetObjectCommand } = require('@aws-sdk/client-s3')
const { randomBytes } = require('crypto')

const s3 = new S3Client({ region: process.env.AWS_REGION })
const BUCKET = process.env.SHARE_BUCKET
const MAX_CODE_SIZE = 32768

function generateSlug() {
  // 暗号論的に安全な乱数で 6 文字 a-z0-9 slug を生成
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789'
  return Array.from(randomBytes(6), b => chars[b % chars.length]).join('')
}

async function findFreeSlug() {
  for (let attempt = 0; attempt < 5; attempt++) {
    const slug = generateSlug()
    try {
      await s3.send(new GetObjectCommand({ Bucket: BUCKET, Key: `shares/${slug}.fav` }))
      // オブジェクトが存在 → 衝突 → 次の試行へ
    } catch (err) {
      // NoSuchKey = スロットが空き → 使用可能
      if (err.name === 'NoSuchKey' || err.$metadata?.httpStatusCode === 404) {
        return slug
      }
      // S3 障害・IAM エラー等は再スローして 500 に伝播させる
      throw err
    }
  }
  throw new Error('slug generation failed after 5 attempts')
}

const CORS_HEADERS = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET,POST,OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
}

exports.handler = async (event) => {
  const method = event.httpMethod || event.requestContext?.http?.method

  if (method === 'OPTIONS') {
    return { statusCode: 200, headers: CORS_HEADERS, body: '' }
  }

  if (method === 'POST') {
    let body
    try {
      body = JSON.parse(event.body || '{}')
    } catch {
      return { statusCode: 400, headers: CORS_HEADERS, body: JSON.stringify({ error: 'invalid json' }) }
    }

    const code = body.code ?? ''
    if (!code || typeof code !== 'string') {
      return { statusCode: 400, headers: CORS_HEADERS, body: JSON.stringify({ error: 'code is required' }) }
    }
    if (Buffer.byteLength(code, 'utf8') > MAX_CODE_SIZE) {
      return { statusCode: 400, headers: CORS_HEADERS, body: JSON.stringify({ error: 'code too large (max 32KB)' }) }
    }

    const slug = await findFreeSlug()
    await s3.send(new PutObjectCommand({
      Bucket: BUCKET,
      Key: `shares/${slug}.fav`,
      Body: code,
      ContentType: 'text/plain; charset=utf-8',
    }))

    const url = `https://play.favnir.dev/playground?s=${slug}`
    return {
      statusCode: 201,
      headers: { ...CORS_HEADERS, 'Content-Type': 'application/json' },
      body: JSON.stringify({ slug, url }),
    }
  }

  if (method === 'GET') {
    const slug = event.pathParameters?.slug
    if (!slug || !/^[a-z0-9]{6}$/.test(slug)) {
      return { statusCode: 400, headers: CORS_HEADERS, body: JSON.stringify({ error: 'invalid slug' }) }
    }
    try {
      const resp = await s3.send(new GetObjectCommand({ Bucket: BUCKET, Key: `shares/${slug}.fav` }))
      const code = await resp.Body.transformToString('utf-8')
      return {
        statusCode: 200,
        headers: { ...CORS_HEADERS, 'Content-Type': 'application/json' },
        body: JSON.stringify({ code }),
      }
    } catch {
      return { statusCode: 404, headers: CORS_HEADERS, body: JSON.stringify({ error: 'not found' }) }
    }
  }

  return { statusCode: 405, headers: CORS_HEADERS, body: JSON.stringify({ error: 'method not allowed' }) }
}
