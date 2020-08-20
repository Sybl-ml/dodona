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

  .btn-primary {
    background-color: ${({ theme }) => theme.mid};
    border: 0.2rem solid ${({ theme }) => theme.mid};;

    &:hover {
      background-color: ${({ theme }) => theme.text};
      color: ${({ theme }) => theme.body};
    }
    
    &:active {
      background-color: ${({ theme }) => theme.body};
    }

    &:focus {
      outline:none;
    }
  }

  .outline{
    background-color:transparent;
    color: ${({ theme }) => theme.mid};
  }
  
  .navbar {
    min-height: 4rem;
    background-color:${({ theme }) => theme.body};
    opacity:0.9;
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
       color:${({ theme }) => theme.mid};
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

  .square {
    position: absolute;
    top: -5%;
    left: 45%;
    width: 10%;
    height: 10%;
    filter: drop-shadow(0 1px 2px 0 0 rgba(0,0,0,.5));
    background-color:${({ theme }) => theme.body};
    transform: rotate(45deg);
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

  .highlight{
    padding-bottom:2rem;
    text-align: center;
    background-color: ${({ theme }) => theme.highlight};
  }

  .highlight-text{
    color: ${({ theme }) => theme.dark};
  }

  `