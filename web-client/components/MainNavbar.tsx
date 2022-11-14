import {Navbar, NavbarGroup, NavbarHeading, NavbarDivider, Button, Classes} from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import Link from 'next/link';
import styles from '../styles/MainNavbar.module.css';

const MainNavbar = () => {
  const {t} = useTranslation('common');

  return (
    <Navbar className={styles.navbar}>

      <NavbarGroup align='left'>
        <NavbarHeading>
          <Link href='/'> Deputy</Link>
        </NavbarHeading>
        <NavbarDivider/>
      </NavbarGroup>
      <input className={`bp4-input ${styles.searchbox}`} type='search' placeholder={t('searchbox')} dir='auto'/>
      <NavbarGroup align='right'>
        <Link href='/packages'>{t('browseAllPackages')}</Link>
        <NavbarDivider/>
        <Link href='/'>{t('logIn')}</Link>
      </NavbarGroup>

    </Navbar>
  );
};

export default MainNavbar;
