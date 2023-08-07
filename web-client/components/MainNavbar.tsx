import {
  Navbar,
  NavbarGroup,
  NavbarHeading,
  NavbarDivider,
  Button,
} from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import Link from 'next/link';
import { useSession, signIn, signOut } from 'next-auth/react';
import styles from '../styles/MainNavbar.module.css';

const MainNavbar = () => {
  const { t } = useTranslation('common');
  const { data: session } = useSession();

  const loginComponent = session ? (
    <>
      <b>{session.user?.name}</b>
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
