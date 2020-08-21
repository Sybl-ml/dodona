import { createGlobalStyle} from "styled-components"


import kollektif from '../fonts/kollektif/Kollektif.ttf';


export const GlobalStyles = createGlobalStyle`
  @font-face {
    font-family: kollektif;
    src: url(${kollektif}) format('truetype');
    font-weight: normal;
    font-style: normal;
  }

  body {
    background: ${({ theme }) => theme.body};
    color: ${({ theme }) => theme.text};
    font-family: kollektif, sans-serif;
    text-align: center;
  }

  .navbar {
    min-height: 4rem;
    background: linear-gradient(${({ theme }) => theme.body}, ${({ theme }) => theme.body} 90%, transparent 100%);
    
    transition: all 0.25s linear;
  }

  .navbar .navbar-nav .nav-link.active{
    color: ${({ theme }) => theme.text};
  }

  .navbar .navbar-nav .show>.nav-link{
    color: ${({ theme }) => theme.text};
  }

  .navbar .navbar-nav .nav-link{
    color: ${({ theme }) => theme.text};
    font-size:1.2rem;
    &:hover {
       color:${({ theme }) => theme.dark};
    } 
  }
  .dropdown-menu{
    background-color:${({ theme }) => theme.body};
    border: none;
		-webkit-filter: drop-shadow(0 2px 2px  rgba(0,0,0,.5));
		-moz-filter: drop-shadow(0 2px 2px  rgba(0,0,0,.5));
		-ms-filter: drop-shadow(0 2px 2px  rgba(0,0,0,.5));
		-o-filter: drop-shadow(0 2px 2px  rgba(0,0,0,.5));
    filter: drop-shadow(0 2px 2px  rgba(0,0,0,.5));
  }

  .dropdown-item{
    color: ${({ theme }) => theme.text};
    &:hover {
      background-color:${({ theme }) => theme.mid};
   } 
   &:active {
     background-color: ${({ theme }) => theme.secondary};
   }
  }

  a {
    color: ${({ theme }) => theme.mid};
    &:hover{
      color: ${({ theme }) => theme.dark};
    }
  }

  `