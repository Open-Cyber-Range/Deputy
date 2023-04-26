interface Preview {
  type: 'picture' | 'video' | 'code';
  value: string[];
}

interface Content {
  type: 'vm' | 'condition' | 'inject' | 'event';
  preview: Preview[];
}

export interface Project {
  content: Content;
}
