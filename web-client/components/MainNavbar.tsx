import {
  Navbar,
  NavbarGroup,
  NavbarHeading,
  NavbarDivider,
  Button,
  Menu,
  MenuItem,
  Popover,
  Position,
} from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import Link from 'next/link';
import { useEffect } from 'react';
import { useRouter } from 'next/router';
import { useSession, signIn, signOut } from 'next-auth/react';
import SearchBar from './SearchBar';

const UserMenu = () => {
  const { t } = useTranslation('common');
  const router = useRouter();

  return (
    <Menu>
      <MenuItem
        text={t('tokens')}
        onClick={() => {
          router.push('/tokens');
        }}
      />
    </Menu>
  );
};

const MainNavbar = () => {
  const { t } = useTranslation('common');
  const { data: session, update } = useSession();

  useEffect(() => {
    const interval = setInterval(() => {
      update();
    }, 1000 * 50);

    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const visibilityHandler = () => {
      if (document.visibilityState === 'visible') {
        update();
      }
    };
    window.addEventListener('visibilitychange', visibilityHandler, false);
    return () =>
      window.removeEventListener('visibilitychange', visibilityHandler, false);
  }, []);

  const loginComponent = session ? (
    <>
      {session.user?.name && (
        <Popover content={<UserMenu />} position={Position.BOTTOM}>
          <Button
            className="font-bolt"
            text={session.user.name}
            small
            icon="caret-down"
          />
        </Popover>
      )}
      <Button
        className="ml-2"
        icon="log-out"
        small
        onClick={(e) => {
          e.preventDefault();
          signOut();
        }}
      >
        {t('logOut')}
      </Button>
    </>
  ) : (
    <Button
      className="font-bolt"
      small
      icon="log-in"
      onClick={(e) => {
        e.preventDefault();
        signIn();
      }}
    >
      {t('logIn')}
    </Button>
  );

  return (
    <Navbar className="flex justify-center items-center bg-cr14-dark-blue">
      <div className="flex items-center basis-[50em]">
        <NavbarGroup align="left">
          <NavbarHeading>
            <span className="bp4-navbar-heading text-m font-bold uppercase tracking-wider text-cr14-gray">
              Deputy
            </span>
          </NavbarHeading>
          <NavbarDivider />
        </NavbarGroup>
        <SearchBar />
        <NavbarGroup align="right">
          <Link className="bp4-button bp4-small" href="/packages">
            {t('browseAllPackages')}
          </Link>
          <NavbarDivider />
          {loginComponent}
        </NavbarGroup>
      </div>
    </Navbar>
  );
};

export default MainNavbar;
