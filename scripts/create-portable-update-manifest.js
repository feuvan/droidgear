import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const FORBIDDEN_SIGNATURE_SNIPPETS = [
  'Your file was signed successfully',
  'Public signature:',
  'Make sure to include this into the signature field of your update server',
]

function normalizeLineEndings(value) {
  return value.replace(/\r\n/g, '\n')
}

export function validatePortableSignatureContent(signatureContent) {
  if (!signatureContent) {
    throw new Error('便携版签名内容不能为空')
  }

  const normalizedSignature = normalizeLineEndings(signatureContent).replace(
    /^\uFEFF/,
    ''
  )

  for (const snippet of FORBIDDEN_SIGNATURE_SNIPPETS) {
    if (normalizedSignature.includes(snippet)) {
      throw new Error(
        `便携版签名文件内容不合法：检测到被污染的 signer 控制台输出片段 "${snippet}"`
      )
    }
  }

  if (!normalizedSignature.startsWith('untrusted comment:')) {
    throw new Error(
      '便携版签名文件内容不合法：必须以 "untrusted comment:" 开头'
    )
  }

  if (!normalizedSignature.includes('\ntrusted comment:')) {
    throw new Error(
      '便携版签名文件内容不合法：缺少 minisign 的 trusted comment 行'
    )
  }

  return signatureContent
}

export function encodePortableSignature(signatureContent) {
  validatePortableSignatureContent(signatureContent)
  return Buffer.from(signatureContent, 'utf8').toString('base64')
}

export function buildPortableUpdateManifest({
  version,
  pubDate,
  url,
  signatureContent,
  sha256,
  releaseUrl,
  notes = '',
}) {
  const requiredFields = {
    version,
    pubDate,
    url,
    sha256,
    releaseUrl,
  }

  for (const [fieldName, fieldValue] of Object.entries(requiredFields)) {
    if (!fieldValue || !fieldValue.trim()) {
      throw new Error(`缺少必填字段：${fieldName}`)
    }
  }

  return {
    version,
    notes,
    pub_date: pubDate,
    url,
    signature: encodePortableSignature(signatureContent),
    sha256,
    release_url: releaseUrl,
  }
}

export function readPortableSignatureFile(signatureFilePath) {
  if (!signatureFilePath || !signatureFilePath.trim()) {
    throw new Error('缺少签名文件路径')
  }

  if (!fs.existsSync(signatureFilePath)) {
    throw new Error(`签名文件不存在：${signatureFilePath}`)
  }

  return fs.readFileSync(signatureFilePath, 'utf8')
}

export function writePortableManifest(outputPath, manifest) {
  if (!outputPath || !outputPath.trim()) {
    throw new Error('缺少 manifest 输出路径')
  }

  fs.mkdirSync(path.dirname(outputPath), { recursive: true })
  fs.writeFileSync(outputPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8')
}

function parseCliArgs(argv) {
  const args = {}

  for (let index = 0; index < argv.length; index += 1) {
    const currentArg = argv[index]
    if (!currentArg.startsWith('--')) {
      throw new Error(`不支持的位置参数：${currentArg}`)
    }

    const key = currentArg.slice(2)
    const value = argv[index + 1]

    if (!value || value.startsWith('--')) {
      throw new Error(`参数 ${currentArg} 缺少取值`)
    }

    args[key] = value
    index += 1
  }

  return args
}

function main() {
  const args = parseCliArgs(process.argv.slice(2))
  const signatureFile = args['signature-file']
  const outputPath = args.output
  const signatureContent = readPortableSignatureFile(signatureFile)
  const manifest = buildPortableUpdateManifest({
    version: args.version,
    pubDate: args['pub-date'],
    url: args.url,
    signatureContent,
    sha256: args.sha256,
    releaseUrl: args['release-url'],
    notes: args.notes ?? '',
  })

  writePortableManifest(outputPath, manifest)
  console.log(`已生成便携版更新清单：${outputPath}`)
}

const currentFilePath = fileURLToPath(import.meta.url)
if (process.argv[1] && path.resolve(process.argv[1]) === currentFilePath) {
  try {
    main()
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error))
    process.exit(1)
  }
}
