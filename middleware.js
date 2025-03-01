import { next } from '@vercel/edge';

export const config = {
  matcher: '/',
};

export default function middleware(req) {
  const authorizationHeader = req.headers.get('authorization');

  if (authorizationHeader) {
    const basicAuth = authorizationHeader.split(' ')[1];
    const [user, password] = atob(basicAuth).toString().split(':');
    const isVercel = process.env.VERCEL === '1';

    if (isVercel) {
      if (
        user === process.env.BASIC_USER &&
        password === process.env.BASIC_PASS
      ) {
        return next();
      }
    }
  }

  return new Response('Basic Auth required', {
    status: 401,
    headers: {
      'WWW-Authenticate': 'Basic realm="Secure Area"',
    },
  });
}
