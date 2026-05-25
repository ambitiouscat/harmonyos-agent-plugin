export interface MarkdownBlock {
  type: 'heading' | 'paragraph' | 'code_block' | 'bullet_list';
  level?: number;
  text?: string;
  language?: string;
  items?: string[];
  children?: MarkdownBlock[];
}

export class MarkdownParser {
  static parse(input: string): MarkdownBlock[] {
    const lines = input.split('\n');
    const blocks: MarkdownBlock[] = [];
    let codeLines: string[] = [];
    let codeLang = '';
    let inCode = false;

    for (const line of lines) {
      // Code block fence: ```language ... ```
      if (line.trimStart().startsWith('```')) {
        if (inCode) {
          blocks.push({ type: 'code_block', language: codeLang, text: codeLines.join('\n') });
          codeLines = [];
          inCode = false;
        } else {
          codeLang = line.trimStart().slice(3).trim();
          inCode = true;
        }
        continue;
      }

      if (inCode) {
        codeLines.push(line);
        continue;
      }

      if (line.trim() === '') {
        continue;
      }

      // Heading: ### Title
      const headingMatch = line.match(/^(#{1,6})\s+(.+)/);
      if (headingMatch) {
        blocks.push({ type: 'heading', level: headingMatch[1].length, text: headingMatch[2] });
        continue;
      }

      // Bullet list item: - text
      if (line.trimStart().startsWith('- ')) {
        const items = [
          line.trimStart().slice(2),
        ];
        blocks.push({ type: 'bullet_list', items });
        continue;
      }

      // Numbered list: 1. text
      if (line.trimStart().match(/^\d+\.\s/)) {
        const items = [
          line.trimStart().replace(/^\d+\.\s/, ''),
        ];
        blocks.push({ type: 'bullet_list', items });
        continue;
      }

      // Default: paragraph
      blocks.push({ type: 'paragraph', text: line });
    }

    // Flush unclosed code block
    if (inCode && codeLines.length > 0) {
      blocks.push({ type: 'code_block', language: codeLang, text: codeLines.join('\n') });
    }

    return blocks;
  }
}
