'use client'

import { useState } from 'react'

export default function Counter() {
const [count, setCount] = useState(0)

return (
  <div>
    <h2>Client Interaction</h2>
    <button onClick={() => setCount(count + 1)} type="button">
      Count: {count}
    </button>
  </div>
)
}