import { Router } from 'itty-router';

const router = Router();

router.get('/', () => {
  return new Response('Welcome to Bytes Radar!', {
    headers: { 'content-type': 'text/plain' },
  });
});

router.get('/health', () => {
  return new Response('OK', {
    headers: { 'content-type': 'text/plain' },
  });
});

router.all('*', () => new Response('Not Found', { status: 404 }));

export default {
  fetch: router.handle,
}; 