import {Navbar, NavbarGroup, NavbarHeading, NavbarDivider} from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import Link from 'next/link';
import styles from '../styles/MainNavbar.module.css';

const MainNavbar = () => {
  const {t} = useTranslation('common');

  return (
    <Navbar className={styles.navbar}>
      <div className={styles.navbar_container}>
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
      </div>
    </Navbar>
  );
};

export default MainNavbar;
