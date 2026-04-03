export type PortalAuthMode = 'login' | 'register' | 'forgot-password';

export interface PortalAuthPageProps {
  signIn: (credentials: { email: string; password: string }) => Promise<unknown>;
  register: (payload: { name: string; email: string; password: string }) => Promise<unknown>;
}
