import { mount } from 'svelte'
import BloomApp from './Bloom.svelte'

const app = mount(BloomApp, {
  target: document.getElementById('app')!,
})

export default app
