import './portal.css';

import { useEffect, useState } from 'react';
import { PortalLoginPage, PortalRegisterPage } from 'sdkwork-api-portal-auth';
import {
  clearPortalSessionToken,
  getPortalMe,
  PortalApiError,
  readPortalSessionToken,
} from 'sdkwork-api-portal-sdk';
import { PortalDashboardPage } from 'sdkwork-api-portal-user';
import type { PortalAuthSession, PortalUserProfile } from 'sdkwork-api-types';

type PortalRoute = '/login' | '/register' | '/dashboard';

function normalizeRoute(hash: string): PortalRoute {
  const candidate = hash.replace(/^#/, '') || '/login';
  if (candidate === '/login' || candidate === '/register' || candidate === '/dashboard') {
    return candidate;
  }
  return '/login';
}

function writeRoute(route: PortalRoute): void {
  window.location.hash = route;
}

function PortalBootScreen() {
  return (
    <section className="portal-loading-shell">
      <div className="portal-loading-card">
        <p className="portal-kicker">Portal Bootstrap</p>
        <h1>Restoring workspace access</h1>
        <p className="portal-status">Checking for an existing portal session token.</p>
      </div>
    </section>
  );
}

export function PortalApp() {
  const [route, setRoute] = useState<PortalRoute>(() => normalizeRoute(window.location.hash));
  const [portalUser, setPortalUser] = useState<PortalUserProfile | null>(null);
  const [bootstrapped, setBootstrapped] = useState(false);

  useEffect(() => {
    const handleHashChange = () => {
      setRoute(normalizeRoute(window.location.hash));
    };

    window.addEventListener('hashchange', handleHashChange);
    if (!window.location.hash) {
      writeRoute(readPortalSessionToken() ? '/dashboard' : '/login');
    }

    return () => {
      window.removeEventListener('hashchange', handleHashChange);
    };
  }, []);

  useEffect(() => {
    const token = readPortalSessionToken();
    if (!token) {
      setPortalUser(null);
      setBootstrapped(true);
      if (route === '/dashboard') {
        writeRoute('/login');
      }
      return;
    }

    let cancelled = false;
    void getPortalMe(token)
      .then((user) => {
        if (cancelled) {
          return;
        }
        setPortalUser(user);
        setBootstrapped(true);
      })
      .catch((error) => {
        if (cancelled) {
          return;
        }

        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          setPortalUser(null);
          setBootstrapped(true);
          writeRoute('/login');
          return;
        }

        setPortalUser(null);
        setBootstrapped(true);
      });

    return () => {
      cancelled = true;
    };
  }, [route]);

  function navigate(nextRoute: PortalRoute) {
    if (normalizeRoute(window.location.hash) !== nextRoute) {
      writeRoute(nextRoute);
      return;
    }
    setRoute(nextRoute);
  }

  function handleAuthenticated(session: PortalAuthSession) {
    setPortalUser(session.user);
    navigate('/dashboard');
  }

  function handleLogout() {
    setPortalUser(null);
    navigate('/login');
  }

  if (!bootstrapped) {
    return (
      <div className="portal-app">
        <PortalBootScreen />
      </div>
    );
  }

  if (route === '/register') {
    return (
      <div className="portal-app">
        <PortalRegisterPage
          onAuthenticated={handleAuthenticated}
          onNavigate={(path) => navigate(normalizeRoute(`#${path}`))}
        />
      </div>
    );
  }

  if (route === '/login' || !portalUser) {
    return (
      <div className="portal-app">
        <PortalLoginPage
          onAuthenticated={handleAuthenticated}
          onNavigate={(path) => navigate(normalizeRoute(`#${path}`))}
        />
      </div>
    );
  }

  return (
    <div className="portal-app">
      <PortalDashboardPage
        onLogout={handleLogout}
        onNavigate={(path) => navigate(normalizeRoute(`#${path}`))}
      />
    </div>
  );
}
