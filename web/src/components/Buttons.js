import { Button } from "react-bootstrap";
import styled from "styled-components";

export const PrimaryButton = styled(Button)` 
    background-color: ${({ theme }) => theme.mid};
    border: 0.2rem solid ${({ theme }) => theme.mid};
    font-weight: bold;

    &:hover {
      background-color: ${({ theme }) => theme.dark};
      border: 0.2rem solid ${({ theme }) => theme.dark};
    }
    
    &:active {
      background-color: ${({ theme }) => theme.highlight};
      border: 0.2rem solid ${({ theme }) => theme.highlight};
    }

    &:focus {
      outline:none;
    }
  }
`;

export const OutlinedPrimaryButton = styled(PrimaryButton)`
  background-color: transparent;
  color: ${({ theme }) => theme.mid};
`;
