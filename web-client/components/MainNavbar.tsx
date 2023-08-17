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
import { useRouter } from 'next/router';
import { useSession, signIn, signOut } from 'next-auth/react';
import styles from '../styles/MainNavbar.module.css';

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
  const { data: session } = useSession();

  const loginComponent = session ? (
    <>
      {session.user?.name && (
        <Popover content={<UserMenu />} position={Position.BOTTOM}>
          <Button text={session.user.name} minimal icon="caret-down" />
        </Popover>
      )}
      <Button
        className="ml-2"
        minimal
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
      minimal
      onClick={(e) => {
        e.preventDefault();
        signIn();
      }}
    >
      {t('logIn')}
    </Button>
  );

  return (
    <Navbar className={styles.navbar}>
      <div className={styles.navbar_container}>
        <NavbarGroup align="left">
          <NavbarHeading>
            <Link href="/"> Deputy</Link>
          </NavbarHeading>
          <NavbarDivider />
        </NavbarGroup>
        <input
          className={`bp4-input ${styles.searchbox}`}
          type="search"
          placeholder={t('searchbox')}
          dir="auto"
        />
        <NavbarGroup align="right">
          <Link href="/packages">{t('browseAllPackages')}</Link>
          <NavbarDivider />
          {loginComponent}
        </NavbarGroup>
      </div>
    </Navbar>
  );
};

export default MainNavbar;
