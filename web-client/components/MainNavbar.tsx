import {Navbar, NavbarGroup, NavbarHeading, NavbarDivider, Button, Classes} from '@blueprintjs/core';
import Link from 'next/link';

const MainNavbar = () => (
  <Navbar>

    <NavbarGroup align='left'>
      <NavbarHeading>
        <Link href='/'> Deputy</Link>
      </NavbarHeading>
      <NavbarDivider/>
    </NavbarGroup>

    <NavbarGroup align='right'>
      <Link href='/packages'>Browse All Packages</Link>
      <NavbarDivider/>
      <Link href='/'>Log In</Link>
    </NavbarGroup>

  </Navbar>
);
export default MainNavbar;
