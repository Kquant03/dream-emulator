# Asset Pipeline Documentation

## Table of Contents
- [Overview](#overview)
- [Supported Asset Types](#supported-asset-types)
- [Asset Import Process](#asset-import-process)
- [Asset Organization](#asset-organization)
- [Optimization](#optimization)
- [Asset Metadata](#asset-metadata)
- [Texture Atlasing](#texture-atlasing)
- [Audio Processing](#audio-processing)
- [Asset Bundles](#asset-bundles)
- [Best Practices](#best-practices)

## Overview

The Dream Emulator asset pipeline handles importing, processing, organizing, and optimizing game assets. It's designed to be artist-friendly while providing powerful optimization features for production games.

### Pipeline Flow
```
Raw Asset → Import → Validation → Processing → Optimization → Storage → Runtime
    ↓          ↓          ↓            ↓             ↓           ↓         ↓
  File      Format    Security    Conversion    Compress    Database   Cache
```

## Supported Asset Types

### Images
| Format | Use Case | Import Settings |
|--------|----------|-----------------|
| PNG | Sprites, UI, Transparency | Lossless, Alpha support |
| JPG | Backgrounds, Photos | Lossy, No alpha |
| GIF | Simple animations | Frame extraction |
| WebP | Modern compression | Fallback to PNG |
| SVG | Scalable UI elements | Rasterize on import |

### Audio
| Format | Use Case | Processing |
|--------|----------|------------|
| MP3 | Music, Long sounds | Streaming |
| OGG | Sound effects | Compressed |
| WAV | High quality SFX | Convert to OGG |
| M4A | iOS compatibility | Convert to MP3 |

### Fonts
| Format | Support | Features |
|--------|---------|----------|
| TTF | Full | Subsetting, Atlas generation |
| OTF | Full | Variable fonts supported |
| WOFF/WOFF2 | Web | Direct use |

### Data
| Format | Use Case | Processing |
|--------|----------|------------|
| JSON | Game data, Levels | Validation, Minification |
| XML | Legacy data | Convert to JSON |
| CSV | Spreadsheet data | Parse to JSON |
| YAML | Configuration | Convert to JSON |

## Asset Import Process

### 1. Drag & Drop Import
```typescript
interface ImportOptions {
  // Automatic detection
  autoDetectType: boolean
  
  // Override settings
  forceType?: AssetType
  
  // Processing options
  generateMipmaps: boolean
  generateColliders: boolean
  extractAnimations: boolean
}

class AssetImporter {
  async import(files: File[], options: ImportOptions) {
    for (const file of files) {
      // Validate file
      await this.validate(file)
      
      // Process based on type
      const processor = this.getProcessor(file.type)
      const processed = await processor.process(file, options)
      
      // Store in database
      await this.store(processed)
      
      // Generate preview
      await this.generatePreview(processed)
    }
  }
}
```

### 2. Validation Pipeline
```typescript
class AssetValidator {
  validators = [
    new FileTypeValidator(),
    new FileSizeValidator({ maxSize: 100 * 1024 * 1024 }), // 100MB
    new ContentValidator(), // Scan for malicious content
    new DimensionValidator({ maxWidth: 4096, maxHeight: 4096 }),
    new FormatValidator()
  ]
  
  async validate(file: File): Promise<ValidationResult> {
    for (const validator of this.validators) {
      const result = await validator.validate(file)
      if (!result.valid) {
        return result
      }
    }
    return { valid: true }
  }
}
```

### 3. Processing Pipeline

#### Image Processing
```typescript
class ImageProcessor {
  async process(file: File, options: ImageImportOptions) {
    const image = await this.decode(file)
    
    // Generate mipmaps for 3D
    if (options.generateMipmaps) {
      await this.generateMipmaps(image)
    }
    
    // Extract sprite sheet
    if (options.isSpriteSheet) {
      return this.extractSprites(image, options.gridSize)
    }
    
    // Optimize
    const optimized = await this.optimize(image, {
      quality: options.quality || 0.9,
      format: options.format || 'webp'
    })
    
    return {
      id: generateId(),
      name: file.name,
      type: 'image',
      data: optimized,
      metadata: {
        width: image.width,
        height: image.height,
        format: optimized.format,
        size: optimized.size
      }
    }
  }
}
```

#### Audio Processing
```typescript
class AudioProcessor {
  async process(file: File, options: AudioImportOptions) {
    const audio = await this.decode(file)
    
    // Normalize volume
    if (options.normalize) {
      audio.normalize()
    }
    
    // Trim silence
    if (options.trimSilence) {
      audio.trimSilence({
        threshold: -40, // dB
        padding: 0.1 // seconds
      })
    }
    
    // Convert format
    const converted = await audio.convert({
      format: 'ogg',
      bitrate: options.quality === 'high' ? 192 : 128,
      channels: options.mono ? 1 : 2
    })
    
    // Generate waveform preview
    const waveform = await this.generateWaveform(audio)
    
    return {
      id: generateId(),
      name: file.name,
      type: 'audio',
      data: converted,
      metadata: {
        duration: audio.duration,
        sampleRate: audio.sampleRate,
        channels: audio.channels,
        waveform
      }
    }
  }
}
```

## Asset Organization

### Directory Structure
```
assets/
├── sprites/
│   ├── characters/
│   │   ├── player/
│   │   └── enemies/
│   ├── environment/
│   └── ui/
├── audio/
│   ├── music/
│   ├── sfx/
│   └── voice/
├── fonts/
├── data/
│   ├── levels/
│   ├── dialogs/
│   └── config/
└── bundles/
    ├── level1.bundle
    └── level2.bundle
```

### Tagging System
```typescript
interface AssetTags {
  // Automatic tags
  type: AssetType
  format: string
  size: 'small' | 'medium' | 'large'
  
  // User tags
  custom: string[]
  
  // Smart tags
  colors?: string[] // Dominant colors
  content?: string[] // AI-detected content
}

class TagManager {
  async autoTag(asset: Asset) {
    const tags: AssetTags = {
      type: asset.type,
      format: asset.format,
      size: this.categorizeSize(asset.size),
      custom: []
    }
    
    // Extract colors from images
    if (asset.type === 'image') {
      tags.colors = await this.extractColors(asset)
    }
    
    // AI content detection
    if (this.aiEnabled) {
      tags.content = await this.detectContent(asset)
    }
    
    return tags
  }
}
```

### Search & Filter
```typescript
class AssetSearch {
  index: SearchIndex
  
  async search(query: string, filters?: SearchFilters) {
    // Full-text search
    let results = await this.index.search(query)
    
    // Apply filters
    if (filters) {
      results = results.filter(asset => {
        if (filters.type && asset.type !== filters.type) return false
        if (filters.tags && !filters.tags.every(t => asset.tags.includes(t))) return false
        if (filters.size && asset.size > filters.size.max) return false
        return true
      })
    }
    
    // Sort by relevance
    return results.sort((a, b) => b.score - a.score)
  }
}
```

## Optimization

### Texture Optimization
```typescript
class TextureOptimizer {
  async optimize(texture: Texture, target: Platform) {
    const optimized = {
      ...texture
    }
    
    // Platform-specific formats
    switch (target) {
      case 'web':
        optimized.format = 'webp'
        optimized.fallback = 'png'
        break
      case 'mobile':
        optimized.format = 'etc2' // Or PVRTC for iOS
        optimized.maxSize = 2048
        break
      case 'desktop':
        optimized.format = 'dxt5'
        optimized.maxSize = 4096
        break
    }
    
    // Resize if needed
    if (texture.width > optimized.maxSize) {
      optimized.data = await this.resize(texture.data, optimized.maxSize)
    }
    
    // Compress
    optimized.data = await this.compress(optimized.data, {
      format: optimized.format,
      quality: this.getQuality(texture)
    })
    
    return optimized
  }
}
```

### Memory Management
```typescript
class AssetMemoryManager {
  private cache = new Map<string, WeakRef<Asset>>()
  private usage = new Map<string, number>()
  private limit = 500 * 1024 * 1024 // 500MB
  private current = 0
  
  async load(id: string): Promise<Asset> {
    // Check cache
    const cached = this.cache.get(id)?.deref()
    if (cached) {
      this.usage.set(id, Date.now())
      return cached
    }
    
    // Load from disk
    const asset = await this.loadFromDisk(id)
    
    // Check memory limit
    if (this.current + asset.size > this.limit) {
      await this.evictLRU()
    }
    
    // Cache
    this.cache.set(id, new WeakRef(asset))
    this.current += asset.size
    this.usage.set(id, Date.now())
    
    return asset
  }
  
  private async evictLRU() {
    // Sort by last usage
    const sorted = Array.from(this.usage.entries())
      .sort((a, b) => a[1] - b[1])
    
    // Evict oldest 25%
    const toEvict = Math.floor(sorted.length * 0.25)
    for (let i = 0; i < toEvict; i++) {
      const [id] = sorted[i]
      const asset = this.cache.get(id)?.deref()
      if (asset) {
        this.current -= asset.size
        this.cache.delete(id)
        this.usage.delete(id)
      }
    }
  }
}
```

## Asset Metadata

### Metadata Schema
```typescript
interface AssetMetadata {
  // Core metadata
  id: string
  name: string
  type: AssetType
  format: string
  size: number
  hash: string
  
  // Import metadata
  importedAt: Date
  importedBy: string
  importSettings: ImportOptions
  
  // File metadata
  originalName: string
  originalSize: number
  modifiedAt: Date
  
  // Type-specific metadata
  typeMetadata: ImageMetadata | AudioMetadata | FontMetadata
  
  // Usage tracking
  usage: {
    scenes: string[]
    prefabs: string[]
    scripts: string[]
    lastUsed: Date
    useCount: number
  }
  
  // Relationships
  dependencies: string[]
  dependents: string[]
}

interface ImageMetadata {
  width: number
  height: number
  channels: number
  hasAlpha: boolean
  isAnimated: boolean
  frames?: number
  colorSpace: 'srgb' | 'linear'
  averageColor: string
  dominantColors: string[]
}
```

### Metadata Extraction
```typescript
class MetadataExtractor {
  async extract(file: File): Promise<AssetMetadata> {
    const base = {
      id: generateId(),
      name: sanitizeName(file.name),
      type: detectType(file),
      format: getExtension(file.name),
      size: file.size,
      hash: await calculateHash(file),
      importedAt: new Date(),
      originalName: file.name,
      originalSize: file.size,
      modifiedAt: new Date(file.lastModified)
    }
    
    // Type-specific extraction
    const typeMetadata = await this.extractTypeMetadata(file, base.type)
    
    return {
      ...base,
      typeMetadata,
      usage: {
        scenes: [],
        prefabs: [],
        scripts: [],
        lastUsed: new Date(),
        useCount: 0
      },
      dependencies: [],
      dependents: []
    }
  }
}
```

## Texture Atlasing

### Automatic Atlas Generation
```typescript
class TextureAtlasGenerator {
  async generate(textures: Texture[], options: AtlasOptions) {
    // Sort by size for better packing
    const sorted = textures.sort((a, b) => 
      (b.width * b.height) - (a.width * a.height)
    )
    
    // Pack using max rects algorithm
    const packer = new MaxRectsPacker(
      options.maxWidth || 2048,
      options.maxHeight || 2048,
      options.padding || 2
    )
    
    const atlas = {
      id: generateId(),
      name: options.name || 'atlas',
      textures: new Map<string, AtlasRegion>()
    }
    
    for (const texture of sorted) {
      const rect = packer.add(
        texture.width + options.padding * 2,
        texture.height + options.padding * 2
      )
      
      if (rect) {
        atlas.textures.set(texture.id, {
          x: rect.x + options.padding,
          y: rect.y + options.padding,
          width: texture.width,
          height: texture.height,
          rotated: rect.rotated
        })
      }
    }
    
    // Render atlas
    const canvas = new OffscreenCanvas(packer.width, packer.height)
    const ctx = canvas.getContext('2d')!
    
    for (const [id, region] of atlas.textures) {
      const texture = textures.find(t => t.id === id)!
      ctx.drawImage(texture.data, region.x, region.y)
    }
    
    atlas.data = await canvas.convertToBlob({ type: 'image/png' })
    return atlas
  }
}
```

### Runtime Atlas Usage
```typescript
class AtlasManager {
  atlases = new Map<string, TextureAtlas>()
  
  getTexture(id: string): AtlasTexture | null {
    for (const [atlasId, atlas] of this.atlases) {
      const region = atlas.regions.get(id)
      if (region) {
        return {
          atlas: atlasId,
          region,
          uvs: this.calculateUVs(region, atlas)
        }
      }
    }
    return null
  }
  
  private calculateUVs(region: AtlasRegion, atlas: TextureAtlas) {
    return {
      u0: region.x / atlas.width,
      v0: region.y / atlas.height,
      u1: (region.x + region.width) / atlas.width,
      v1: (region.y + region.height) / atlas.height
    }
  }
}
```

## Audio Processing

### Audio Sprite Generation
```typescript
class AudioSpriteGenerator {
  async generate(sounds: AudioFile[], options: AudioSpriteOptions) {
    const sprite = {
      id: generateId(),
      name: options.name,
      sprites: new Map<string, AudioSpriteDef>()
    }
    
    // Combine audio files
    const combined = new AudioBuffer({
      numberOfChannels: 2,
      sampleRate: 44100,
      length: 0
    })
    
    let currentTime = 0
    const gap = options.gap || 0.1 // seconds
    
    for (const sound of sounds) {
      const buffer = await this.decode(sound)
      
      sprite.sprites.set(sound.id, {
        start: currentTime,
        end: currentTime + buffer.duration,
        loop: sound.metadata?.loop || false
      })
      
      // Copy audio data
      combined.copyToChannel(buffer, 0, currentTime * combined.sampleRate)
      
      currentTime += buffer.duration + gap
    }
    
    // Encode combined audio
    sprite.data = await this.encode(combined, {
      format: 'ogg',
      quality: options.quality || 0.8
    })
    
    return sprite
  }
}
```

## Asset Bundles

### Bundle Creation
```typescript
interface BundleManifest {
  id: string
  name: string
  version: string
  assets: string[]
  dependencies: string[]
  size: number
  hash: string
}

class BundleBuilder {
  async build(assets: Asset[], options: BundleOptions): Promise<Bundle> {
    const bundle = {
      manifest: {
        id: generateId(),
        name: options.name,
        version: options.version || '1.0.0',
        assets: assets.map(a => a.id),
        dependencies: [],
        size: 0,
        hash: ''
      },
      data: new Map<string, Asset>()
    }
    
    // Process assets
    for (const asset of assets) {
      // Optimize for bundle
      const optimized = await this.optimize(asset, options)
      bundle.data.set(asset.id, optimized)
      bundle.manifest.size += optimized.size
      
      // Track dependencies
      if (asset.dependencies) {
        bundle.manifest.dependencies.push(...asset.dependencies)
      }
    }
    
    // Remove duplicate dependencies
    bundle.manifest.dependencies = [...new Set(bundle.manifest.dependencies)]
    
    // Calculate hash
    bundle.manifest.hash = await this.calculateHash(bundle)
    
    // Compress bundle
    if (options.compress) {
      bundle.data = await this.compress(bundle.data)
    }
    
    return bundle
  }
}
```

### Bundle Loading
```typescript
class BundleLoader {
  loaded = new Map<string, Bundle>()
  loading = new Map<string, Promise<Bundle>>()
  
  async load(bundleId: string): Promise<Bundle> {
    // Check if already loaded
    if (this.loaded.has(bundleId)) {
      return this.loaded.get(bundleId)!
    }
    
    // Check if currently loading
    if (this.loading.has(bundleId)) {
      return this.loading.get(bundleId)!
    }
    
    // Start loading
    const promise = this.loadBundle(bundleId)
    this.loading.set(bundleId, promise)
    
    try {
      const bundle = await promise
      this.loaded.set(bundleId, bundle)
      this.loading.delete(bundleId)
      
      // Load dependencies
      await Promise.all(
        bundle.manifest.dependencies.map(dep => this.load(dep))
      )
      
      return bundle
    } catch (error) {
      this.loading.delete(bundleId)
      throw error
    }
  }
  
  private async loadBundle(bundleId: string): Promise<Bundle> {
    // Fetch bundle
    const response = await fetch(`/bundles/${bundleId}.bundle`)
    const data = await response.arrayBuffer()
    
    // Decompress if needed
    const decompressed = await this.decompress(data)
    
    // Parse bundle
    return this.parseBundle(decompressed)
  }
}
```

## Best Practices

### 1. Asset Naming
- Use descriptive names: `player_walk_01.png` not `img1.png`
- Include version numbers: `enemy_v2.png`
- Use consistent naming schemes
- Avoid special characters and spaces

### 2. Organization
- Create logical folder hierarchies
- Group related assets together
- Use consistent categorization
- Maintain clean project structure

### 3. Optimization
- Import at appropriate resolutions
- Use texture atlases for sprites
- Compress audio appropriately
- Remove unused assets regularly

### 4. Version Control
- Track source assets separately
- Use LFS for large binary files
- Document asset changes
- Maintain asset changelog

### 5. Performance
- Set appropriate import settings
- Use level-of-detail (LOD) assets
- Stream large assets
- Preload critical assets

### 6. Memory Usage
```typescript
// Good: Load on demand
const texture = await assetManager.load('player_sprite')

// Bad: Load everything at once
const allTextures = await assetManager.loadAll('textures/*')

// Good: Unload when done
assetManager.unload('level1/*')

// Good: Use asset bundles
await bundleLoader.load('level2_bundle')
```

### 7. Asset Validation
```typescript
// Validate before production
const validator = new AssetValidator()
const issues = await validator.validateProject()

if (issues.length > 0) {
  console.warn('Asset issues found:', issues)
  // Fix issues before building
}
```

This comprehensive asset pipeline ensures efficient asset management from import to runtime, enabling creators to focus on their games while the system handles optimization and organization.