import React from 'react';
import Image from 'next/image';
import easLogo from 'assets/logos/enterprise-estonia-eas-vector-logo.svg';
import norwayGrantsLogo from 'assets/logos/Norway_grants_White.png';

const NavbarSponsors = () => (
  <>
    <a
      href="https://eeagrants.org/"
      target="_blank"
      rel="noopener noreferrer"
      className="h-full"
    >
      <Image
        src={norwayGrantsLogo}
        alt="norwayGrants-logo"
        className="h-full px-4 object-contain py-1"
      />
    </a>
    <a
      href="https://eas.ee"
      target="_blank"
      rel="noopener noreferrer"
      className="h-full flex items-center"
    >
      <Image src={easLogo} alt="eas-logo" className="h-16 pr-4" />
    </a>
  </>
);

export default NavbarSponsors;
