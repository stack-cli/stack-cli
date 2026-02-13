import { marked } from 'marked'

interface MarkdownPostProps {
    content: string
    title: string
}

export default async function MarkdownPost({ content, title }: MarkdownPostProps) {
    // Process markdown on the server
    marked.setOptions({
        gfm: true,
        breaks: false,
    })
    const htmlContent = await marked.parse(content)

    return (
        <article>
            <h1>{title}</h1>
            <div dangerouslySetInnerHTML={{ __html: htmlContent }} />
        </article>
    )
}