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
  Alignment,
} from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import Link from 'next/link';
import { useEffect } from 'react';
import { useRouter } from 'next/router';
import { useSession, signIn, signOut } from 'next-auth/react';
import Image from 'next/image';
import SearchBar from './SearchBar';
import NavbarSponsors from './SponsorIcons';
import deputylogo from '../assets/logos/DEPUTY_BLUEBLUEv4.svg';

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

  return (
    <Navbar className="flex justify-center items-center bg-cr14-dark-blue">
      <div className="flex justify-between items-center basis-[60rem]">
        <NavbarGroup align={Alignment.LEFT} className="max-w-[20rem]">
          <NavbarHeading>
            <Link className="hover:no-underline focus:outline-none" href="/">
              <span className="py-4">
                <Image
                  className="object-contain h-10 w-10"
                  src={deputylogo}
                  alt="Deputy Logo"
                />
              </span>
            </Link>
          </NavbarHeading>
          <div className="border-r border-white h-5 mx-2" />
          <NavbarSponsors />
          <div className="border-r border-white h-5 mx-2" />
        </NavbarGroup>
        <NavbarGroup
          align="center"
          className="flex justify-between items-center"
        >
          <SearchBar />
          <Link
            className="bp5-button bp5-small bp5-minimal rounded-md  w-full"
            href="/packages"
          >
            <span className="text-white">{t('browseAllPackages')}</span>
          </Link>
        </NavbarGroup>
        <NavbarGroup align="right">
          <NavbarDivider />
          {session ? (
            <>
              {session.user?.name && (
                <Popover
                  usePortal={false}
                  content={<UserMenu />}
                  position={Position.BOTTOM}
                  autoFocus={false}
                >
                  <Button
                    className="font-bolt outline-none"
                    textClassName="text-white whitespace-nowrap rounded-md"
                    small
                    minimal
                    icon="caret-down"
                  >
                    <span className="text-white whitespace-nowrap rounded-md">
                      {session.user.name}
                    </span>
                  </Button>
                </Popover>
              )}
              <Button
                className="ml-2"
                icon="log-out"
                small
                minimal
                onClick={(e) => {
                  e.preventDefault();
                  signOut();
                }}
              >
                <span className="text-white whitespace-nowrap rounded-md">
                  {t('logOut')}
                </span>
              </Button>
            </>
          ) : (
            <Button
              className="font-bolt"
              small
              minimal
              icon="log-in"
              onClick={(e) => {
                e.preventDefault();
                signIn();
              }}
            >
              <span className="text-white whitespace-nowrap rounded-md">
                {t('logIn')}
              </span>
            </Button>
          )}
        </NavbarGroup>
      </div>
    </Navbar>
  );
};

export default MainNavbar;
