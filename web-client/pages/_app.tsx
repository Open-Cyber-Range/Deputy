import type {AppProps} from 'next/app';
import '@blueprintjs/core/lib/css/blueprint.css';
import '../styles/global.css';
import MainNavbar from '../components/MainNavbar';

function MyApp({Component, pageProps}: AppProps) {
  return (
    <>
      <MainNavbar/>
      <Component {...pageProps} />
    </>
  );
}

export default MyApp;
