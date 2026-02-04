// Lucide React icons type declarations
declare module 'lucide-react' {
  export const Search: React.ComponentType<any>;
  export const Settings: React.ComponentType<any>;
  export const LogOut: React.ComponentType<any>;
  export const User: React.ComponentType<any>;
  export const Menu: React.ComponentType<any>;
  export const Image: React.ComponentType<any>;
  export const Video: React.ComponentType<any>;
  export const Share2: React.ComponentType<any>;
  export const Radio: React.ComponentType<any>;
  export const Grid3X3: React.ComponentType<any>;
  export const Heart: React.ComponentType<any>;
  export const Clock: React.ComponentType<any>;
  export const Trash2: React.ComponentType<any>;
  export const Home: React.ComponentType<any>;
  export const Layers: React.ComponentType<any>;
  export const List: React.ComponentType<any>;
  export const Plus: React.ComponentType<any>;
  export const Upload: React.ComponentType<any>;
  export const Filter: React.ComponentType<any>;
  export const X: React.ComponentType<any>;
  export const Check: React.ComponentType<any>;
  export const AlertCircle: React.ComponentType<any>;
  export const FileImage: React.ComponentType<any>;
  export const FileVideo: React.ComponentType<any>;
}

// Hash-WASM type declarations
declare module 'hash-wasm' {
  export interface IHasher {
    update(input: Uint8Array | string): IHasher;
    digest(): string;
  }
  export function createXXHash3(): Promise<IHasher>;
  export function createXXHash2(bits?: number): Promise<IHasher>;
}
