import MarkdownPost from '@/components/MarkdownPost'

const blogPost = `
# Welcome to rari!

This markdown is processed **on the server** using the \`marked\` package.

- Fast server-side rendering
- Universal NPM package support
- Zero configuration required
`

export default function BlogPage() {
return (
  <div>
    <MarkdownPost title="My Blog Post" content={blogPost} />
  </div>
)
}

export const metadata = {
title: 'Blog | React Supabase Example',
description: 'Read our latest posts',
}
