import { Button } from '#/components/ui/button';
import { Input } from '#/components/ui/input';
import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'

export const Route = createFileRoute('/')({ component: App })

function App() {
  const [article, setArticle] = useState("");
  const [url, setUrl] = useState("");

  async function fetchArtcile() {
    let contents = await fetch("http://localhost:8080/reader?url=" + url)
      .then(c => c.json())
    
    setArticle((contents as any).html_content)
  }

  return (
    <>
      <Input value={url} onChange={(e) => setUrl(e.target.value)} />
      <Button onClick={fetchArtcile}>Fetch</Button>

      <div>
        <div className='text-justify' dangerouslySetInnerHTML={{ __html: article }} />
      </div>
      
    </>
  )
}
